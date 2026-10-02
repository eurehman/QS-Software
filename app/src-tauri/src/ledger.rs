use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::format_number;

const ZERO: &str = "0";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantityLedger {
    pub item_id: i64,
    pub original_qty: String,
    pub revised_qty: String,
    pub planned_qty: String,
    pub contract_qty: String,
    pub executed_qty: String,
    pub measured_qty: String,
    pub certified_qty: String,
    pub billed_qty: String,
    pub paid_qty: String,
    pub forecast_qty: String,
    pub final_qty: String,
    pub remaining_qty: String,
}

pub fn ensure_seed(conn: &Connection, item_id: i64) -> Result<(), String> {
    let quantity: String = conn
        .query_row(
            "SELECT quantity FROM work_item WHERE id = ?1",
            params![item_id],
            |row| row.get(0),
        )
        .map_err(|_| "That bill item was not found.".to_string())?;
    let opening = number_text(&quantity)?;
    conn.execute(
        "INSERT INTO quantity_ledger (
            item_id, original_qty, revised_qty, planned_qty, contract_qty,
            executed_qty, measured_qty, certified_qty, billed_qty, paid_qty,
            forecast_qty, final_qty
         ) VALUES (?1, ?2, ?2, ?3, ?2, ?3, ?3, ?3, ?3, ?3, ?3, ?3)
         ON CONFLICT(item_id) DO NOTHING",
        params![item_id, opening, ZERO],
    )
    .map_err(|err| err.to_string())?;
    rollup_parents(conn, item_id)
}

pub fn set_measured(conn: &Connection, item_id: i64, quantity: &str) -> Result<(), String> {
    let measured = number_text(quantity)?;
    let updated = conn
        .execute(
            "UPDATE quantity_ledger SET measured_qty = ?1 WHERE item_id = ?2",
            params![measured, item_id],
        )
        .map_err(|err| err.to_string())?;
    if updated == 0 {
        return Err("That bill item has no quantity ledger.".into());
    }
    rollup_parents(conn, item_id)
}

pub fn post_balance(conn: &Connection, item_id: i64, balance: &str, quantity: &str, username: &str) -> Result<QuantityLedger, String> {
    let column = balance_column(balance)?;
    let posted = number_text(quantity)?;
    let children: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_item WHERE parent_id = ?1",
            params![item_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if children > 0 {
        return Err("A heading balance comes from its children.".into());
    }
    let old: String = conn
        .query_row(
            &format!("SELECT {column} FROM quantity_ledger WHERE item_id = ?1"),
            params![item_id],
            |row| row.get(0),
        )
        .map_err(|_| "That bill item has no quantity ledger.".to_string())?;
    conn.execute(
        &format!("UPDATE quantity_ledger SET {column} = ?1 WHERE item_id = ?2"),
        params![posted, item_id],
    )
    .map_err(|err| err.to_string())?;
    rollup_parents(conn, item_id)?;
    crate::audit::record(
        conn,
        username,
        "edit",
        &format!("bill:{item_id}:{}", balance.trim().to_lowercase()),
        &old,
        &posted,
    )?;
    read(conn, item_id)
}

fn balance_column(balance: &str) -> Result<&'static str, String> {
    let balance = balance.trim().to_lowercase();
    match balance.as_str() {
        "executed" => Ok("executed_qty"),
        "certified" => Ok("certified_qty"),
        "billed" => Ok("billed_qty"),
        _ => Err("Post executed, certified, or billed.".into()),
    }
}

pub fn read(conn: &Connection, item_id: i64) -> Result<QuantityLedger, String> {
    let row = raw(conn, item_id)?.ok_or_else(|| "That bill item has no quantity ledger.".to_string())?;
    Ok(with_remaining(item_id, row))
}

pub fn export_core(conn: &Connection, project_code: &str, path: &std::path::Path) -> Result<(), String> {
    let (code, name): (String, String) = conn
        .query_row(
            "SELECT code, name FROM project WHERE code = ?1",
            params![project_code.trim()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| format!("Project {} was not found.", project_code.trim()))?;
    let mut code_stmt = conn
        .prepare("SELECT kind, code, name FROM code_entry ORDER BY kind, code")
        .map_err(|err| err.to_string())?;
    let codes = code_stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    drop(code_stmt);
    let mut code_rows = vec![vec!["Kind".into(), "Code".into(), "Name".into()]];
    for (kind, entry_code, entry_name) in codes {
        code_rows.push(vec![kind, entry_code, entry_name]);
    }
    let mut bill_rows = vec![vec![
        "Item".into(),
        "Unit".into(),
        "Quantity".into(),
        "Original".into(),
        "Contracted".into(),
        "Measured".into(),
        "Certified".into(),
        "Billed".into(),
        "Remaining".into(),
        "WBS".into(),
        "CBS".into(),
        "Package".into(),
    ]];
    for item in crate::codes::list_items(conn, project_code)? {
        let ledger = read(conn, item.id)?;
        bill_rows.push(vec![
            item.name,
            item.unit_code,
            item.quantity,
            ledger.original_qty,
            ledger.contract_qty,
            ledger.measured_qty,
            ledger.certified_qty,
            ledger.billed_qty,
            ledger.remaining_qty,
            item.wbs_code,
            item.cbs_code,
            item.package_code,
        ]);
    }
    crate::xlsx::write_sheets(
        path,
        &[
            ("Project", vec![vec!["Code".into(), "Name".into(), "Company".into()], vec![code, name, crate::project::load_company(conn)?.name]]),
            ("Codes", code_rows),
            ("Bill", bill_rows),
        ],
    )
}

fn rollup_parents(conn: &Connection, item_id: i64) -> Result<(), String> {
    let mut current: Option<i64> = conn
        .query_row(
            "SELECT parent_id FROM work_item WHERE id = ?1",
            params![item_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    while let Some(parent_id) = current {
        rewrite_parent(conn, parent_id)?;
        current = conn
            .query_row(
                "SELECT parent_id FROM work_item WHERE id = ?1",
                params![parent_id],
                |row| row.get(0),
            )
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn rewrite_parent(conn: &Connection, parent_id: i64) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id FROM work_item WHERE parent_id = ?1 ORDER BY id")
        .map_err(|err| err.to_string())?;
    let children = stmt
        .query_map(params![parent_id], |row| row.get::<_, i64>(0))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let mut sums = [0.0; 11];
    for child_id in children {
        if let Some(row) = raw(conn, child_id)? {
            for (index, value) in row.iter().enumerate() {
                sums[index] += parse_number(value)?;
            }
        }
    }
    let text: Vec<String> = sums.iter().copied().map(format_number).collect();
    conn.execute(
        "INSERT INTO quantity_ledger (
            item_id, original_qty, revised_qty, planned_qty, contract_qty,
            executed_qty, measured_qty, certified_qty, billed_qty, paid_qty,
            forecast_qty, final_qty
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(item_id) DO UPDATE SET
            original_qty = excluded.original_qty,
            revised_qty = excluded.revised_qty,
            planned_qty = excluded.planned_qty,
            contract_qty = excluded.contract_qty,
            executed_qty = excluded.executed_qty,
            measured_qty = excluded.measured_qty,
            certified_qty = excluded.certified_qty,
            billed_qty = excluded.billed_qty,
            paid_qty = excluded.paid_qty,
            forecast_qty = excluded.forecast_qty,
            final_qty = excluded.final_qty",
        params![
            parent_id, text[0], text[1], text[2], text[3], text[4], text[5], text[6], text[7], text[8], text[9], text[10]
        ],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn raw(conn: &Connection, item_id: i64) -> Result<Option<[String; 11]>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT original_qty, revised_qty, planned_qty, contract_qty,
                    executed_qty, measured_qty, certified_qty, billed_qty,
                    paid_qty, forecast_qty, final_qty
             FROM quantity_ledger WHERE item_id = ?1",
        )
        .map_err(|err| err.to_string())?;
    let mut rows = stmt
        .query(params![item_id])
        .map_err(|err| err.to_string())?;
    let Some(row) = rows.next().map_err(|err| err.to_string())? else {
        return Ok(None);
    };
    Ok(Some([
        row.get(0).map_err(|err| err.to_string())?,
        row.get(1).map_err(|err| err.to_string())?,
        row.get(2).map_err(|err| err.to_string())?,
        row.get(3).map_err(|err| err.to_string())?,
        row.get(4).map_err(|err| err.to_string())?,
        row.get(5).map_err(|err| err.to_string())?,
        row.get(6).map_err(|err| err.to_string())?,
        row.get(7).map_err(|err| err.to_string())?,
        row.get(8).map_err(|err| err.to_string())?,
        row.get(9).map_err(|err| err.to_string())?,
        row.get(10).map_err(|err| err.to_string())?,
    ]))
}

fn with_remaining(item_id: i64, row: [String; 11]) -> QuantityLedger {
    let original = parse_number(&row[0]).unwrap_or(0.0);
    let certified = parse_number(&row[6]).unwrap_or(0.0);
    let billed = parse_number(&row[7]).unwrap_or(0.0);
    let used = certified.max(billed);
    QuantityLedger {
        item_id,
        original_qty: row[0].clone(),
        revised_qty: row[1].clone(),
        planned_qty: row[2].clone(),
        contract_qty: row[3].clone(),
        executed_qty: row[4].clone(),
        measured_qty: row[5].clone(),
        certified_qty: row[6].clone(),
        billed_qty: row[7].clone(),
        paid_qty: row[8].clone(),
        forecast_qty: row[9].clone(),
        final_qty: row[10].clone(),
        remaining_qty: format_number(original - used),
    }
}

fn number_text(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(ZERO.into());
    }
    Ok(format_number(parse_number(raw)?))
}

fn parse_number(raw: &str) -> Result<f64, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(0.0);
    }
    raw.parse::<f64>().map_err(|_| format!("{raw} is not a number."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::{save_code, save_item, set_item_quantity};
    use crate::db::create_firm;
    use crate::project::save_project;

    #[test]
    fn concrete_opening_balances_are_100() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        let heading = save_item(
            &firm.conn,
            "TWR",
            None,
            1,
            "Structure",
            "",
            "",
            wbs.id,
            cbs.id,
            cost.id,
            unit.id,
        )
        .unwrap();
        let item = save_item(
            &firm.conn,
            "TWR",
            Some(heading.id),
            1,
            "Concrete — Structural",
            "100",
            "1",
            wbs.id,
            cbs.id,
            cost.id,
            unit.id,
        )
        .unwrap();
        let ledger = read(&firm.conn, item.id).unwrap();
        assert_eq!(ledger.original_qty, "100");
        assert_eq!(ledger.revised_qty, "100");
        assert_eq!(ledger.planned_qty, "0");
        assert_eq!(ledger.contract_qty, "100");
        assert_eq!(ledger.executed_qty, "0");
        assert_eq!(ledger.measured_qty, "0");
        assert_eq!(ledger.certified_qty, "0");
        assert_eq!(ledger.billed_qty, "0");
        assert_eq!(ledger.paid_qty, "0");
        assert_eq!(ledger.forecast_qty, "0");
        assert_eq!(ledger.final_qty, "0");
        assert_eq!(ledger.remaining_qty, "100");
        let parent = read(&firm.conn, heading.id).unwrap();
        assert_eq!(parent.original_qty, "100");
        assert_eq!(parent.contract_qty, "100");
        assert_eq!(parent.remaining_qty, "100");

        set_item_quantity(&firm.conn, item.id, "35").unwrap();
        let after = read(&firm.conn, item.id).unwrap();
        assert_eq!(after.original_qty, "100");
        assert_eq!(after.contract_qty, "100");
        assert_eq!(after.measured_qty, "35");
        assert_eq!(after.certified_qty, "0");
        assert_eq!(after.billed_qty, "0");
        assert_eq!(after.remaining_qty, "100");
    }

    #[test]
    fn core_workbook_shows_100_cubic_metres() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        save_item(
            &firm.conn,
            "TWR",
            None,
            1,
            "Concrete — Structural",
            "100",
            "1",
            wbs.id,
            cbs.id,
            cost.id,
            unit.id,
        )
        .unwrap();
        let path = dir.path().join("core.xlsx");
        export_core(&firm.conn, "TWR", &path).unwrap();
        let bill = crate::xlsx::read_sheet(&path, "Bill").unwrap();
        let row = bill
            .iter()
            .find(|row| row.first().is_some_and(|cell| cell == "Concrete — Structural"))
            .unwrap();
        assert!(row.contains(&"100".to_string()));
        assert!(row.contains(&"m3".to_string()));
        let project = crate::xlsx::read_sheet(&path, "Project").unwrap();
        assert_eq!(project[1][0], "TWR");
    }

    #[test]
    fn posted_quantities_reconcile() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        let heading = save_item(
            &firm.conn, "TWR", None, 1, "Structure", "", "", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        let item = save_item(
            &firm.conn, "TWR", Some(heading.id), 1, "Concrete — Structural", "100", "1",
            wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();

        let opening = read(&firm.conn, item.id).unwrap();
        assert_eq!(opening.original_qty, "100");
        assert_eq!(opening.executed_qty, "0");
        assert_eq!(opening.certified_qty, "0");
        assert_eq!(opening.billed_qty, "0");
        assert_eq!(opening.remaining_qty, "100");

        let executed = post_balance(&firm.conn, item.id, "executed", "40", "qs").unwrap();
        assert_eq!(executed.original_qty, "100");
        assert_eq!(executed.executed_qty, "40");
        assert_eq!(executed.measured_qty, "0");
        assert_eq!(executed.certified_qty, "0");
        assert_eq!(executed.billed_qty, "0");
        assert_eq!(executed.remaining_qty, "100");

        let certified = post_balance(&firm.conn, item.id, "certified", "35", "qs").unwrap();
        assert_eq!(certified.original_qty, "100");
        assert_eq!(certified.executed_qty, "40");
        assert_eq!(certified.certified_qty, "35");
        assert_eq!(certified.remaining_qty, "65");

        let billed = post_balance(&firm.conn, item.id, "billed", "35", "qs").unwrap();
        assert_eq!(billed.billed_qty, "35");
        assert_eq!(billed.certified_qty, "35");
        assert_eq!(billed.remaining_qty, "65");
        assert_eq!(billed.original_qty, "100");

        let more = post_balance(&firm.conn, item.id, "billed", "40", "qs").unwrap();
        assert_eq!(more.billed_qty, "40");
        assert_eq!(more.certified_qty, "35");
        assert_eq!(more.remaining_qty, "60");
        assert_eq!(more.contract_qty, "100");

        let parent = read(&firm.conn, heading.id).unwrap();
        assert_eq!(parent.original_qty, "100");
        assert_eq!(parent.executed_qty, "40");
        assert_eq!(parent.certified_qty, "35");
        assert_eq!(parent.billed_qty, "40");
        assert_eq!(parent.remaining_qty, "60");

        let heading_error = post_balance(&firm.conn, heading.id, "executed", "1", "qs").unwrap_err();
        assert!(heading_error.contains("children"));
        let unknown = post_balance(&firm.conn, item.id, "paid", "1", "qs").unwrap_err();
        assert!(unknown.contains("executed"));

        let edit = crate::audit::list_events(&firm.conn)
            .unwrap()
            .into_iter()
            .find(|event| event.target == format!("bill:{}:executed", item.id))
            .unwrap();
        assert_eq!(edit.username, "qs");
        assert_eq!(edit.old_value, "0");
        assert_eq!(edit.new_value, "40");
    }
}

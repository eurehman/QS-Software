use rusqlite::{params, Connection};
use serde::Serialize;

const KINDS: &[&str] = &["wbs", "cbs", "cost", "unit", "work", "discipline", "package"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEntry {
    pub id: i64,
    pub kind: String,
    pub code: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItem {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub version_no: i64,
    pub name: String,
    pub quantity: String,
    pub rate: String,
    pub amount: String,
    pub wbs_code: String,
    pub cbs_code: String,
    pub cost_code: String,
    pub unit_code: String,
    pub work_code: String,
    pub discipline_code: String,
    pub package_code: String,
}

pub fn save_code(conn: &Connection, kind: &str, code: &str, name: &str) -> Result<CodeEntry, String> {
    save_code_as(conn, kind, code, name, "")
}

pub fn save_code_as(conn: &Connection, kind: &str, code: &str, name: &str, username: &str) -> Result<CodeEntry, String> {
    let kind = kind.trim();
    let code = code.trim();
    let name = name.trim();
    if !KINDS.contains(&kind) {
        return Err("Choose WBS, CBS, cost, unit, work, discipline, or package.".into());
    }
    if code.is_empty() || name.is_empty() {
        return Err("Enter a code and a name.".into());
    }
    let previous = match conn.query_row(
        "SELECT name FROM code_entry WHERE kind = ?1 AND code = ?2",
        params![kind, code],
        |row| row.get::<_, String>(0),
    ) {
        Ok(stored) => Some(stored),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(err) => return Err(err.to_string()),
    };
    conn.execute(
        "INSERT INTO code_entry (kind, code, name) VALUES (?1, ?2, ?3)
         ON CONFLICT(kind, code) DO UPDATE SET name = excluded.name",
        params![kind, code, name],
    )
    .map_err(|err| err.to_string())?;
    let old = previous.clone().unwrap_or_default();
    let action = if previous.is_none() { "create" } else { "edit" };
    crate::audit::record(conn, username, action, &format!("code:{kind}:{code}"), &old, name)?;
    conn.query_row(
        "SELECT id, kind, code, name FROM code_entry WHERE kind = ?1 AND code = ?2",
        params![kind, code],
        |row| {
            Ok(CodeEntry {
                id: row.get(0)?,
                kind: row.get(1)?,
                code: row.get(2)?,
                name: row.get(3)?,
            })
        },
    )
    .map_err(|err| err.to_string())
}

pub fn list_codes(conn: &Connection, kind: &str) -> Result<Vec<CodeEntry>, String> {
    let kind = kind.trim();
    if !KINDS.contains(&kind) {
        return Err("Choose WBS, CBS, cost, unit, work, discipline, or package.".into());
    }
    let mut stmt = conn
        .prepare("SELECT id, kind, code, name FROM code_entry WHERE kind = ?1 ORDER BY code")
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![kind], |row| {
            Ok(CodeEntry {
                id: row.get(0)?,
                kind: row.get(1)?,
                code: row.get(2)?,
                name: row.get(3)?,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|err| err.to_string())
}

pub fn save_item(
    conn: &Connection,
    project_code: &str,
    parent_id: Option<i64>,
    version_no: i64,
    name: &str,
    quantity: &str,
    rate: &str,
    wbs_id: i64,
    cbs_id: i64,
    cost_id: i64,
    unit_id: i64,
) -> Result<WorkItem, String> {
    save_classified(
        conn,
        project_code,
        parent_id,
        version_no,
        name,
        quantity,
        rate,
        wbs_id,
        cbs_id,
        cost_id,
        unit_id,
        None,
        None,
        None,
        "",
    )
}

pub fn save_classified(
    conn: &Connection,
    project_code: &str,
    parent_id: Option<i64>,
    version_no: i64,
    name: &str,
    quantity: &str,
    rate: &str,
    wbs_id: i64,
    cbs_id: i64,
    cost_id: i64,
    unit_id: i64,
    work_id: Option<i64>,
    discipline_id: Option<i64>,
    package_id: Option<i64>,
    username: &str,
) -> Result<WorkItem, String> {
    let name = name.trim();
    let quantity = quantity.trim();
    let rate = rate.trim();
    if name.is_empty() {
        return Err("Enter an item name.".into());
    }
    if version_no < 1 {
        return Err("Enter a version number of 1 or more.".into());
    }
    let project_id: i64 = conn
        .query_row(
            "SELECT id FROM project WHERE code = ?1",
            params![project_code.trim()],
            |row| row.get(0),
        )
        .map_err(|_| format!("Project {} was not found.", project_code.trim()))?;
    if let Some(parent_id) = parent_id {
        let (parent_project, parent_version): (i64, i64) = conn
            .query_row(
                "SELECT project_id, version_no FROM work_item WHERE id = ?1",
                params![parent_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| "That parent item was not found.".to_string())?;
        if parent_project != project_id {
            return Err("The parent item belongs to another project.".into());
        }
        if parent_version != version_no {
            return Err("Use a parent from the same version.".into());
        }
    }
    expect_kind(conn, wbs_id, "wbs")?;
    expect_kind(conn, cbs_id, "cbs")?;
    expect_kind(conn, cost_id, "cost")?;
    expect_kind(conn, unit_id, "unit")?;
    let work_id = optional_kind(conn, work_id, "work")?;
    let discipline_id = optional_kind(conn, discipline_id, "discipline")?;
    let package_id = optional_kind(conn, package_id, "package")?;
    let amount = line_amount(quantity, rate)?;
    conn.execute(
        "INSERT INTO work_item
           (project_id, parent_id, version_no, name, quantity, rate, amount,
            wbs_id, cbs_id, cost_id, unit_id, work_id, discipline_id, package_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            project_id, parent_id, version_no, name, quantity, rate, amount,
            wbs_id, cbs_id, cost_id, unit_id, work_id, discipline_id, package_id
        ],
    )
    .map_err(|err| err.to_string())?;
    let id = conn.last_insert_rowid();
    refresh_amounts(conn, id)?;
    crate::ledger::ensure_seed(conn, id)?;
    crate::audit::record(conn, username, "create", &format!("bill:{id}"), "", name)?;
    crate::audit::record(conn, username, "create", &format!("bill:{id}:quantity"), "", quantity)?;
    list_items(conn, project_code)?
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| "The item was saved but could not be read back.".into())
}

pub fn list_items(conn: &Connection, project_code: &str) -> Result<Vec<WorkItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT item.id, item.parent_id, item.version_no, item.name,
                    item.quantity, item.rate, item.amount,
                    wbs.code, cbs.code, cost.code, unit.code,
                    COALESCE(work.code, ''), COALESCE(discipline.code, ''), COALESCE(package.code, '')
             FROM work_item item
             JOIN project ON project.id = item.project_id
             JOIN code_entry wbs ON wbs.id = item.wbs_id
             JOIN code_entry cbs ON cbs.id = item.cbs_id
             JOIN code_entry cost ON cost.id = item.cost_id
             JOIN code_entry unit ON unit.id = item.unit_id
             LEFT JOIN code_entry work ON work.id = item.work_id
             LEFT JOIN code_entry discipline ON discipline.id = item.discipline_id
             LEFT JOIN code_entry package ON package.id = item.package_id
             WHERE project.code = ?1
             ORDER BY item.version_no, item.id",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![project_code.trim()], |row| {
            Ok(WorkItem {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                version_no: row.get(2)?,
                name: row.get(3)?,
                quantity: row.get(4)?,
                rate: row.get(5)?,
                amount: row.get(6)?,
                wbs_code: row.get(7)?,
                cbs_code: row.get(8)?,
                cost_code: row.get(9)?,
                unit_code: row.get(10)?,
                work_code: row.get(11)?,
                discipline_code: row.get(12)?,
                package_code: row.get(13)?,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|err| err.to_string())
}

pub fn revise_quantity(conn: &Connection, item_id: i64, quantity: &str, username: &str) -> Result<(), String> {
    let quantity = quantity.trim();
    parse_number(quantity)?;
    let children: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_item WHERE parent_id = ?1",
            params![item_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if children > 0 {
        return Err("A heading quantity comes from its children.".into());
    }
    let old: String = conn
        .query_row("SELECT quantity FROM work_item WHERE id = ?1", params![item_id], |row| row.get(0))
        .map_err(|_| "That bill item was not found.".to_string())?;
    conn.execute(
        "UPDATE work_item SET quantity = ?1 WHERE id = ?2",
        params![quantity, item_id],
    )
    .map_err(|err| err.to_string())?;
    refresh_amounts(conn, item_id)?;
    crate::audit::record(conn, username, "edit", &format!("bill:{item_id}:quantity"), &old, quantity)
}

pub(crate) fn set_item_quantity(conn: &Connection, item_id: i64, quantity: &str) -> Result<(), String> {
    let children: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_item WHERE parent_id = ?1",
            params![item_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if children > 0 {
        return Err("A heading quantity comes from its children.".into());
    }
    crate::ledger::ensure_seed(conn, item_id)?;
    conn.execute(
        "UPDATE work_item SET quantity = ?1 WHERE id = ?2",
        params![quantity.trim(), item_id],
    )
    .map_err(|err| err.to_string())?;
    refresh_amounts(conn, item_id)?;
    crate::ledger::set_measured(conn, item_id, quantity)
}

pub(crate) fn set_item_rate(conn: &Connection, item_id: i64, rate: &str) -> Result<(), String> {
    let children: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_item WHERE parent_id = ?1",
            params![item_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if children > 0 {
        return Err("A heading amount comes from its children.".into());
    }
    conn.execute(
        "UPDATE work_item SET rate = ?1 WHERE id = ?2",
        params![rate.trim(), item_id],
    )
    .map_err(|err| err.to_string())?;
    refresh_amounts(conn, item_id)
}

fn line_amount(quantity: &str, rate: &str) -> Result<String, String> {
    if quantity.is_empty() && rate.is_empty() {
        return Ok("0".into());
    }
    let quantity = parse_number(quantity)?;
    let rate = parse_number(rate)?;
    Ok(format_number(quantity * rate))
}

fn parse_number(raw: &str) -> Result<f64, String> {
    if raw.is_empty() {
        return Err("Enter both quantity and rate.".into());
    }
    raw.parse::<f64>().map_err(|_| format!("{raw} is not a number."))
}

pub(crate) fn format_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }
    let text = format!("{value:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn refresh_amounts(conn: &Connection, start_id: i64) -> Result<(), String> {
    let mut current = Some(start_id);
    while let Some(id) = current {
        let mut stmt = conn
            .prepare("SELECT amount FROM work_item WHERE parent_id = ?1")
            .map_err(|err| err.to_string())?;
        let children = stmt
            .query_map(params![id], |row| row.get::<_, String>(0))
            .map_err(|err| err.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;
        let amount = if children.is_empty() {
            let (quantity, rate): (String, String) = conn
                .query_row(
                    "SELECT quantity, rate FROM work_item WHERE id = ?1",
                    params![id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|err| err.to_string())?;
            line_amount(&quantity, &rate)?
        } else {
            let sum = children
                .iter()
                .map(|value| parse_number(value))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .sum();
            format_number(sum)
        };
        conn.execute("UPDATE work_item SET amount = ?1 WHERE id = ?2", params![amount, id])
            .map_err(|err| err.to_string())?;
        current = conn
            .query_row("SELECT parent_id FROM work_item WHERE id = ?1", params![id], |row| row.get(0))
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn optional_kind(conn: &Connection, id: Option<i64>, kind: &str) -> Result<Option<i64>, String> {
    if let Some(id) = id {
        expect_kind(conn, id, kind)?;
        Ok(Some(id))
    } else {
        Ok(None)
    }
}

fn expect_kind(conn: &Connection, id: i64, kind: &str) -> Result<(), String> {
    let found: String = conn
        .query_row("SELECT kind FROM code_entry WHERE id = ?1", params![id], |row| row.get(0))
        .map_err(|_| format!("Choose a {kind} code."))?;
    if found == kind {
        Ok(())
    } else {
        Err(format!("That code is not a {kind} code."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_firm;
    use crate::project::save_project;

    #[test]
    fn codes_are_reusable_on_items() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        save_item(&firm.conn, "TWR", None, 1, "Slab", "1", "1", wbs.id, cbs.id, cost.id, unit.id).unwrap();
        save_item(&firm.conn, "TWR", None, 1, "Beam", "1", "1", wbs.id, cbs.id, cost.id, unit.id).unwrap();
        let items = list_items(&firm.conn, "TWR").unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.unit_code == "m3" && item.wbs_code == "03.10"));
        let units = list_codes(&firm.conn, "unit").unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].id, unit.id);
    }

    #[test]
    fn amount_matches_excel_quantity_times_rate() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        let heading = save_item(
            &firm.conn, "TWR", None, 1, "Concrete works", "", "", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        let slab = save_item(
            &firm.conn, "TWR", Some(heading.id), 1, "Slab", "2.5", "10", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        save_item(
            &firm.conn, "TWR", Some(heading.id), 1, "Beam", "1", "20", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        assert_eq!(slab.amount, "25");
        let items = list_items(&firm.conn, "TWR").unwrap();
        let parent = items.iter().find(|item| item.id == heading.id).unwrap();
        assert_eq!(parent.amount, "45");
        save_item(
            &firm.conn, "TWR", Some(heading.id), 2, "Slab", "3", "10", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap_err();
        let revised = save_item(
            &firm.conn, "TWR", None, 2, "Slab", "3", "10", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        assert_eq!(revised.amount, "30");
        let original = list_items(&firm.conn, "TWR").unwrap();
        assert_eq!(original.iter().find(|item| item.id == slab.id).unwrap().amount, "25");
        assert_eq!(original.iter().filter(|item| item.version_no == 1).count(), 3);
    }

    #[test]
    fn two_items_share_a_work_package() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        let work = save_code(&firm.conn, "work", "C31", "Concreting").unwrap();
        let discipline = save_code(&firm.conn, "discipline", "CIV", "Civil").unwrap();
        let package = save_code(&firm.conn, "package", "CW-01", "Concrete works").unwrap();
        save_classified(
            &firm.conn, "TWR", None, 1, "Slab", "1", "1",
            wbs.id, cbs.id, cost.id, unit.id, Some(work.id), Some(discipline.id), Some(package.id), "",
        )
        .unwrap();
        save_classified(
            &firm.conn, "TWR", None, 1, "Beam", "1", "1",
            wbs.id, cbs.id, cost.id, unit.id, Some(work.id), Some(discipline.id), Some(package.id), "",
        )
        .unwrap();
        let items = list_items(&firm.conn, "TWR").unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.package_code == "CW-01" && item.discipline_code == "CIV" && item.work_code == "C31"));
        assert_eq!(list_codes(&firm.conn, "package").unwrap().len(), 1);
    }
}

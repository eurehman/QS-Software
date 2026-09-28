use std::collections::BTreeMap;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::format_number;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileLine {
    pub name: String,
    pub amount_a: String,
    pub amount_b: String,
    pub difference: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reconciliation {
    pub total_a: String,
    pub total_b: String,
    pub difference: String,
    pub approved: bool,
    pub lines: Vec<ReconcileLine>,
}

pub fn reconcile(conn: &Connection, project_code: &str, version_a: i64, version_b: i64) -> Result<Reconciliation, String> {
    if version_a < 1 || version_b < 1 {
        return Err("Enter a version number of 1 or more.".into());
    }
    if version_a == version_b {
        return Err("Choose two different versions.".into());
    }
    let project_id = project_id(conn, project_code)?;
    let left = root_amounts(conn, project_id, version_a)?;
    let right = root_amounts(conn, project_id, version_b)?;
    let mut names = BTreeMap::<String, ()>::new();
    for name in left.keys().chain(right.keys()) {
        names.insert(name.clone(), ());
    }
    let mut lines = Vec::new();
    let mut line_difference = 0.0;
    for name in names.keys() {
        let amount_a = left.get(name).copied().unwrap_or(0.0);
        let amount_b = right.get(name).copied().unwrap_or(0.0);
        let difference = amount_b - amount_a;
        line_difference += difference;
        lines.push(ReconcileLine {
            name: name.clone(),
            amount_a: format_number(amount_a),
            amount_b: format_number(amount_b),
            difference: format_number(difference),
        });
    }
    let total_a: f64 = left.values().sum();
    let total_b: f64 = right.values().sum();
    let difference = total_b - total_a;
    if format_number(line_difference) != format_number(difference) {
        return Err("The line differences do not reconcile to the bill totals.".into());
    }
    let approved: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM boq_approval WHERE project_id = ?1 AND version_no = ?2",
            params![project_id, version_b],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    Ok(Reconciliation {
        total_a: format_number(total_a),
        total_b: format_number(total_b),
        difference: format_number(difference),
        approved: approved > 0,
        lines,
    })
}

pub fn approve_boq(conn: &Connection, project_code: &str, version_no: i64) -> Result<(), String> {
    if version_no < 1 {
        return Err("Enter a version number of 1 or more.".into());
    }
    let project_id = project_id(conn, project_code)?;
    conn.execute(
        "INSERT INTO boq_approval (project_id, version_no) VALUES (?1, ?2)
         ON CONFLICT(project_id, version_no) DO NOTHING",
        params![project_id, version_no],
    )
    .map(|_| ())
    .map_err(|err| err.to_string())
}

fn root_amounts(conn: &Connection, project_id: i64, version_no: i64) -> Result<BTreeMap<String, f64>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT name, amount FROM work_item
             WHERE project_id = ?1 AND version_no = ?2 AND parent_id IS NULL",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![project_id, version_no], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let mut amounts = BTreeMap::new();
    for (name, amount) in rows {
        let total = amounts.entry(name).or_insert(0.0);
        *total += number(&amount)?;
    }
    Ok(amounts)
}

fn project_id(conn: &Connection, project_code: &str) -> Result<i64, String> {
    conn.query_row("SELECT id FROM project WHERE code = ?1", params![project_code.trim()], |row| row.get(0))
        .map_err(|_| format!("Project {} was not found.", project_code.trim()))
}

fn number(raw: &str) -> Result<f64, String> {
    raw.trim().parse().map_err(|_| format!("{raw} is not a number."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::{save_code, save_item};
    use crate::db::create_firm;
    use crate::project::save_project;

    #[test]
    fn two_versions_reconcile() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        let codes = (wbs.id, cbs.id, cost.id, unit.id);
        save_item(&firm.conn, "TWR", None, 1, "Slab", "2.5", "10", codes.0, codes.1, codes.2, codes.3).unwrap();
        save_item(&firm.conn, "TWR", None, 1, "Beam", "1", "20", codes.0, codes.1, codes.2, codes.3).unwrap();
        save_item(&firm.conn, "TWR", None, 2, "Slab", "3", "10", codes.0, codes.1, codes.2, codes.3).unwrap();
        save_item(&firm.conn, "TWR", None, 2, "Beam", "1", "20", codes.0, codes.1, codes.2, codes.3).unwrap();
        save_item(&firm.conn, "TWR", None, 2, "Extra", "1", "10", codes.0, codes.1, codes.2, codes.3).unwrap();
        let compared = reconcile(&firm.conn, "TWR", 1, 2).unwrap();
        assert_eq!(compared.total_a, "45");
        assert_eq!(compared.total_b, "60");
        assert_eq!(compared.difference, "15");
        assert!(!compared.approved);
        let line_sum: f64 = compared.lines.iter().map(|line| line.difference.parse::<f64>().unwrap()).sum();
        assert_eq!(format_number(line_sum), compared.difference);
        approve_boq(&firm.conn, "TWR", 2).unwrap();
        let approved = reconcile(&firm.conn, "TWR", 1, 2).unwrap();
        assert!(approved.approved);
        assert_eq!(approved.difference, "15");
    }
}

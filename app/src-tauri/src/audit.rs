use std::collections::HashMap;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::backup::utc_stamp;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    pub id: i64,
    pub created_at: String,
    pub username: String,
    pub action: String,
    pub target: String,
    pub old_value: String,
    pub new_value: String,
}

pub fn save_cells(conn: &Connection, username: &str, cells: &[(i64, i64, String)]) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|err| err.to_string())?;
    let previous = load_cells(&tx)?;
    let mut next: HashMap<(i64, i64), String> = HashMap::new();
    for (row, col, raw) in cells {
        if raw.is_empty() || *row < 0 || *col < 0 {
            continue;
        }
        next.insert((*row, *col), raw.clone());
    }
    let when = utc_stamp();
    for (key, old) in &previous {
        if !next.contains_key(key) {
            insert_event(&tx, &when, username, "delete", &target(*key), old, "")?;
        }
    }
    for (key, new_value) in &next {
        match previous.get(key) {
            None => insert_event(&tx, &when, username, "create", &target(*key), "", new_value)?,
            Some(old) if old != new_value => {
                insert_event(&tx, &when, username, "edit", &target(*key), old, new_value)?;
            }
            Some(_) => {}
        }
    }
    tx.execute("DELETE FROM sheet_cell", []).map_err(|err| err.to_string())?;
    for ((row, col), raw) in &next {
        tx.execute(
            "INSERT INTO sheet_cell (sheet_row, sheet_col, raw) VALUES (?1, ?2, ?3)",
            params![row, col, raw],
        )
        .map_err(|err| err.to_string())?;
    }
    tx.commit().map_err(|err| err.to_string())
}

pub fn list_events(conn: &Connection) -> Result<Vec<AuditEvent>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, created_at, username, action, target, old_value, new_value
             FROM audit_event ORDER BY id DESC LIMIT 100",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AuditEvent {
                id: row.get(0)?,
                created_at: row.get(1)?,
                username: row.get(2)?,
                action: row.get(3)?,
                target: row.get(4)?,
                old_value: row.get(5)?,
                new_value: row.get(6)?,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|err| err.to_string())
}

fn load_cells(conn: &Connection) -> Result<HashMap<(i64, i64), String>, String> {
    let mut stmt = conn
        .prepare("SELECT sheet_row, sheet_col, raw FROM sheet_cell")
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let key = (row.get::<_, i64>(0)?, row.get::<_, i64>(1)?);
            let raw: String = row.get(2)?;
            Ok((key, raw))
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<HashMap<_, _>, _>>().map_err(|err| err.to_string())
}

pub fn record(
    conn: &Connection,
    username: &str,
    action: &str,
    target: &str,
    old_value: &str,
    new_value: &str,
) -> Result<(), String> {
    if old_value == new_value {
        return Ok(());
    }
    insert_event(conn, &utc_stamp(), username, action, target, old_value, new_value)
}

fn insert_event(
    conn: &Connection,
    when: &str,
    username: &str,
    action: &str,
    target: &str,
    old_value: &str,
    new_value: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO audit_event (created_at, username, action, target, old_value, new_value)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![when, username, action, target, old_value, new_value],
    )
    .map(|_| ())
    .map_err(|err| err.to_string())
}

fn target(key: (i64, i64)) -> String {
    format!("sheet:{}:{}", key.0, key.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_firm;

    #[test]
    fn an_edit_leaves_a_history_row() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_cells(&firm.conn, "admin", &[(0, 0, "Concrete".into())]).unwrap();
        save_cells(&firm.conn, "admin", &[(0, 0, "Steel".into())]).unwrap();
        let events = list_events(&firm.conn).unwrap();
        let edit = events.iter().find(|event| event.action == "edit").unwrap();
        assert_eq!(edit.username, "admin");
        assert_eq!(edit.target, "sheet:0:0");
        assert_eq!(edit.old_value, "Concrete");
        assert_eq!(edit.new_value, "Steel");
        assert!(!edit.created_at.is_empty());
        save_cells(&firm.conn, "admin", &[(0, 0, "Steel".into())]).unwrap();
        assert_eq!(list_events(&firm.conn).unwrap().len(), events.len());
    }

    #[test]
    fn bill_quantity_edit_leaves_a_history_row() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        crate::project::save_project_as(&firm.conn, "TWR", "Tower site", "qs").unwrap();
        crate::project::save_project_as(&firm.conn, "TWR", "Harbour tower", "qs").unwrap();
        let wbs = crate::codes::save_code_as(&firm.conn, "wbs", "03.10", "Concrete", "qs").unwrap();
        crate::codes::save_code_as(&firm.conn, "wbs", "03.10", "Structural concrete", "qs").unwrap();
        let cbs = crate::codes::save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = crate::codes::save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = crate::codes::save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        let item = crate::codes::save_item(
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
        crate::codes::revise_quantity(&firm.conn, item.id, "80", "qs").unwrap();
        let events = list_events(&firm.conn).unwrap();
        let edit = events
            .iter()
            .find(|event| event.action == "edit" && event.target == format!("bill:{}:quantity", item.id))
            .unwrap();
        assert_eq!(edit.username, "qs");
        assert_eq!(edit.old_value, "100");
        assert_eq!(edit.new_value, "80");
        assert!(!edit.created_at.is_empty());
        let project = events.iter().find(|event| event.target == "project:TWR" && event.action == "edit").unwrap();
        assert_eq!(project.username, "qs");
        assert_eq!(project.old_value, "Tower site");
        assert_eq!(project.new_value, "Harbour tower");
        let code = events
            .iter()
            .find(|event| event.target == "code:wbs:03.10" && event.action == "edit")
            .unwrap();
        assert_eq!(code.old_value, "Concrete");
        assert_eq!(code.new_value, "Structural concrete");
        let ledger = crate::ledger::read(&firm.conn, item.id).unwrap();
        assert_eq!(ledger.original_qty, "100");
        let stored = crate::codes::list_items(&firm.conn, "TWR").unwrap();
        assert_eq!(stored[0].quantity, "80");
        let before = list_events(&firm.conn).unwrap().len();
        crate::codes::revise_quantity(&firm.conn, item.id, "80", "qs").unwrap();
        assert_eq!(list_events(&firm.conn).unwrap().len(), before);
    }
}

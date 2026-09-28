use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::{self, format_number};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasureLine {
    pub id: i64,
    pub item_id: i64,
    pub description: String,
    pub times: String,
    pub length: String,
    pub width: String,
    pub height: String,
    pub quantity: String,
}

pub fn add_line(
    conn: &Connection,
    item_id: i64,
    description: &str,
    times: &str,
    length: &str,
    width: &str,
    height: &str,
) -> Result<MeasureLine, String> {
    let description = description.trim();
    if description.is_empty() {
        return Err("Enter a measurement description.".into());
    }
    line_quantity(times, length, width, height)?;
    conn.execute(
        "INSERT INTO measure_line (item_id, description, times, length, width, height)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![item_id, description, times.trim(), length.trim(), width.trim(), height.trim()],
    )
    .map_err(|err| err.to_string())?;
    let id = conn.last_insert_rowid();
    rollup(conn, item_id)?;
    list_lines(conn, item_id)?
        .into_iter()
        .find(|line| line.id == id)
        .ok_or_else(|| "The measurement was saved but could not be read back.".into())
}

pub fn list_lines(conn: &Connection, item_id: i64) -> Result<Vec<MeasureLine>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, item_id, description, times, length, width, height
             FROM measure_line WHERE item_id = ?1 ORDER BY id",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![item_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|err| err.to_string())?;
    rows.map(|row| {
        let (id, item_id, description, times, length, width, height) = row.map_err(|err| err.to_string())?;
        Ok(MeasureLine {
            quantity: line_quantity(&times, &length, &width, &height)?,
            id,
            item_id,
            description,
            times,
            length,
            width,
            height,
        })
    })
    .collect()
}

pub fn sheet_total(conn: &Connection, item_id: i64) -> Result<String, String> {
    let lines = list_lines(conn, item_id)?;
    let total = lines
        .iter()
        .map(|line| line.quantity.parse::<f64>().map_err(|err| err.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum();
    Ok(format_number(total))
}

fn rollup(conn: &Connection, item_id: i64) -> Result<(), String> {
    let total = sheet_total(conn, item_id)?;
    codes::set_item_quantity(conn, item_id, &total)
}

fn line_quantity(times: &str, length: &str, width: &str, height: &str) -> Result<String, String> {
    let mut product = 1.0;
    let mut used = false;
    for raw in [times, length, width, height] {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let value: f64 = raw.parse().map_err(|_| format!("{raw} is not a number."))?;
        product *= value;
        used = true;
    }
    if !used {
        return Err("Enter at least one dimension.".into());
    }
    Ok(format_number(product))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::{list_items, save_code, save_item};
    use crate::db::create_firm;
    use crate::project::save_project;

    #[test]
    fn sheet_total_equals_measured_quantity() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        let slab = save_item(
            &firm.conn, "TWR", None, 1, "Slab", "0", "10", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        add_line(&firm.conn, slab.id, "Bay A", "2", "2.5", "2", "").unwrap();
        add_line(&firm.conn, slab.id, "Bay B", "1", "5", "", "").unwrap();
        assert_eq!(sheet_total(&firm.conn, slab.id).unwrap(), "15");
        let stored = list_items(&firm.conn, "TWR").unwrap();
        let item = stored.iter().find(|item| item.id == slab.id).unwrap();
        assert_eq!(item.quantity, "15");
        assert_eq!(item.amount, "150");
    }
}

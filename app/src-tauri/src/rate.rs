use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::{self, format_number};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateBuildup {
    pub item_id: i64,
    pub material_qty: String,
    pub material_rate: String,
    pub labour_qty: String,
    pub labour_rate: String,
    pub plant_qty: String,
    pub plant_rate: String,
    pub wastage_percent: String,
    pub overhead_percent: String,
    pub profit_percent: String,
    pub composite_rate: String,
}

pub fn save_buildup(
    conn: &Connection,
    item_id: i64,
    material_qty: &str,
    material_rate: &str,
    labour_qty: &str,
    labour_rate: &str,
    plant_qty: &str,
    plant_rate: &str,
    wastage_percent: &str,
    overhead_percent: &str,
    profit_percent: &str,
) -> Result<RateBuildup, String> {
    let composite = composite_rate(
        material_qty,
        material_rate,
        labour_qty,
        labour_rate,
        plant_qty,
        plant_rate,
        wastage_percent,
        overhead_percent,
        profit_percent,
    )?;
    conn.execute(
        "INSERT INTO rate_buildup (
           item_id, material_qty, material_rate, labour_qty, labour_rate, plant_qty, plant_rate,
           wastage_percent, overhead_percent, profit_percent, composite_rate
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(item_id) DO UPDATE SET
           material_qty = excluded.material_qty,
           material_rate = excluded.material_rate,
           labour_qty = excluded.labour_qty,
           labour_rate = excluded.labour_rate,
           plant_qty = excluded.plant_qty,
           plant_rate = excluded.plant_rate,
           wastage_percent = excluded.wastage_percent,
           overhead_percent = excluded.overhead_percent,
           profit_percent = excluded.profit_percent,
           composite_rate = excluded.composite_rate",
        params![
            item_id,
            material_qty.trim(),
            material_rate.trim(),
            labour_qty.trim(),
            labour_rate.trim(),
            plant_qty.trim(),
            plant_rate.trim(),
            wastage_percent.trim(),
            overhead_percent.trim(),
            profit_percent.trim(),
            composite
        ],
    )
    .map_err(|err| err.to_string())?;
    codes::set_item_rate(conn, item_id, &composite)?;
    load(conn, item_id)
}

pub fn load(conn: &Connection, item_id: i64) -> Result<RateBuildup, String> {
    conn.query_row(
        "SELECT item_id, material_qty, material_rate, labour_qty, labour_rate, plant_qty, plant_rate,
                wastage_percent, overhead_percent, profit_percent, composite_rate
         FROM rate_buildup WHERE item_id = ?1",
        params![item_id],
        |row| {
            Ok(RateBuildup {
                item_id: row.get(0)?,
                material_qty: row.get(1)?,
                material_rate: row.get(2)?,
                labour_qty: row.get(3)?,
                labour_rate: row.get(4)?,
                plant_qty: row.get(5)?,
                plant_rate: row.get(6)?,
                wastage_percent: row.get(7)?,
                overhead_percent: row.get(8)?,
                profit_percent: row.get(9)?,
                composite_rate: row.get(10)?,
            })
        },
    )
    .map_err(|_| "This item has no rate analysis.".into())
}

fn composite_rate(
    material_qty: &str,
    material_rate: &str,
    labour_qty: &str,
    labour_rate: &str,
    plant_qty: &str,
    plant_rate: &str,
    wastage_percent: &str,
    overhead_percent: &str,
    profit_percent: &str,
) -> Result<String, String> {
    let material = number(material_qty)? * number(material_rate)?;
    let labour = number(labour_qty)? * number(labour_rate)?;
    let plant = number(plant_qty)? * number(plant_rate)?;
    let wastage = material * number(wastage_percent)? / 100.0;
    let direct = material + labour + plant + wastage;
    let overhead = direct * number(overhead_percent)? / 100.0;
    let profit = (direct + overhead) * number(profit_percent)? / 100.0;
    Ok(format_number(direct + overhead + profit))
}

fn number(raw: &str) -> Result<f64, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Enter every rate-analysis figure. Use 0 when a component is not used.".into());
    }
    raw.parse().map_err(|_| format!("{raw} is not a number."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::{list_items, save_code, save_item};
    use crate::db::create_firm;
    use crate::project::save_project;

    #[test]
    fn composite_rate_matches_the_worked_sheet() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        let item = save_item(
            &firm.conn, "TWR", None, 1, "Slab", "1", "0", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        let buildup = save_buildup(
            &firm.conn, item.id, "8", "100", "2", "50", "1", "40", "5", "10", "10",
        )
        .unwrap();
        assert_eq!(buildup.composite_rate, "1185.8");
        let stored = list_items(&firm.conn, "TWR").unwrap();
        let slab = stored.iter().find(|row| row.id == item.id).unwrap();
        assert_eq!(slab.rate, "1185.8");
        assert_eq!(slab.amount, "1185.8");
    }
}

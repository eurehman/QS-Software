use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::format_number;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub id: i64,
    pub kind: String,
    pub version_no: i64,
    pub area: String,
    pub rate: String,
    pub total: String,
}

pub fn save_estimate(
    conn: &Connection,
    project_code: &str,
    kind: &str,
    version_no: i64,
    area: &str,
    rate: &str,
) -> Result<Estimate, String> {
    let kind = kind.trim();
    if kind != "boq" && kind != "area" {
        return Err("Choose a BOQ estimate or an area-rate estimate.".into());
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
    let total = if kind == "boq" {
        boq_total(conn, project_id, version_no)?
    } else {
        let area_value = number(area)?;
        let rate_value = number(rate)?;
        format_number(area_value * rate_value)
    };
    conn.execute(
        "INSERT INTO estimate (project_id, kind, version_no, area, rate, total)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![project_id, kind, version_no, area.trim(), rate.trim(), total],
    )
    .map_err(|err| err.to_string())?;
    Ok(Estimate {
        id: conn.last_insert_rowid(),
        kind: kind.to_string(),
        version_no,
        area: area.trim().to_string(),
        rate: rate.trim().to_string(),
        total,
    })
}

pub fn boq_total(conn: &Connection, project_id: i64, version_no: i64) -> Result<String, String> {
    let mut stmt = conn
        .prepare(
            "SELECT amount FROM work_item
             WHERE project_id = ?1 AND version_no = ?2 AND parent_id IS NULL",
        )
        .map_err(|err| err.to_string())?;
    let amounts = stmt
        .query_map(params![project_id, version_no], |row| row.get::<_, String>(0))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let total = amounts
        .iter()
        .map(|amount| number(amount))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum();
    Ok(format_number(total))
}

fn number(raw: &str) -> Result<f64, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Enter the area and the rate.".into());
    }
    raw.parse().map_err(|_| format!("{raw} is not a number."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::{save_code, save_item};
    use crate::db::create_firm;
    use crate::project::save_project;

    #[test]
    fn estimate_total_ties_to_the_boq() {
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
        save_item(
            &firm.conn, "TWR", Some(heading.id), 1, "Slab", "2.5", "10", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        save_item(
            &firm.conn, "TWR", Some(heading.id), 1, "Beam", "1", "20", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        save_item(
            &firm.conn, "TWR", None, 1, "Preliminaries", "1", "10", wbs.id, cbs.id, cost.id, unit.id,
        )
        .unwrap();
        let estimate = save_estimate(&firm.conn, "TWR", "boq", 1, "", "").unwrap();
        assert_eq!(estimate.total, "55");
        let area = save_estimate(&firm.conn, "TWR", "area", 1, "100", "25").unwrap();
        assert_eq!(area.total, "2500");
    }
}

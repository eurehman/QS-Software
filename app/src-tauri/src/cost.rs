use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::format_number;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostPosition {
    pub budget: String,
    pub actual: String,
    pub variance: String,
}

pub fn save_budget(conn: &Connection, project_code: &str, amount: &str) -> Result<CostPosition, String> {
    let budget = number(amount)?;
    let project_id = project_id(conn, project_code)?;
    conn.execute(
        "INSERT INTO cost_budget (project_id, amount) VALUES (?1, ?2)
         ON CONFLICT(project_id) DO UPDATE SET amount = excluded.amount",
        params![project_id, format_number(budget)],
    )
    .map_err(|err| err.to_string())?;
    position(conn, project_id, budget)
}

fn position(conn: &Connection, project_id: i64, budget: f64) -> Result<CostPosition, String> {
    let mut stmt = conn
        .prepare(
            "SELECT ipc.this_bill
             FROM ipc
             JOIN contract ON contract.id = ipc.contract_id
             WHERE contract.project_id = ?1",
        )
        .map_err(|err| err.to_string())?;
    let bills = stmt
        .query_map(params![project_id], |row| row.get::<_, String>(0))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let actual: f64 = bills.iter().map(|bill| number(bill)).collect::<Result<Vec<_>, _>>()?.into_iter().sum();
    Ok(CostPosition {
        budget: format_number(budget),
        actual: format_number(actual),
        variance: format_number(budget - actual),
    })
}

fn project_id(conn: &Connection, project_code: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT id FROM project WHERE code = ?1",
        params![project_code.trim()],
        |row| row.get(0),
    )
    .map_err(|_| format!("Project {} was not found.", project_code.trim()))
}

fn number(raw: &str) -> Result<f64, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Enter a budget.".into());
    }
    raw.parse().map_err(|_| format!("{raw} is not a number."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{save_contract, save_contractor};
    use crate::db::create_firm;
    use crate::ipc::save_certificate;
    use crate::project::save_project;

    #[test]
    fn variance_equals_budget_minus_actual() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        save_certificate(&firm.conn, contract.id, 1, "0", "400", "0", "0", "0").unwrap();
        save_certificate(&firm.conn, contract.id, 2, "400", "1000", "0", "0", "0").unwrap();
        let position = save_budget(&firm.conn, "TWR", "1500").unwrap();
        assert_eq!(position.actual, "1000");
        assert_eq!(position.variance, "500");
    }
}

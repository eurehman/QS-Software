use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::format_number;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractView {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub contractor_name: String,
    pub value: String,
}

pub fn save_contractor(conn: &Connection, name: &str) -> Result<i64, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Enter a contractor name.".into());
    }
    conn.execute(
        "INSERT INTO contractor (name) VALUES (?1)
         ON CONFLICT(name) DO UPDATE SET name = excluded.name",
        params![name],
    )
    .map_err(|err| err.to_string())?;
    conn.query_row("SELECT id FROM contractor WHERE name = ?1", params![name], |row| row.get(0))
        .map_err(|err| err.to_string())
}

pub fn save_contract(
    conn: &Connection,
    project_code: &str,
    contractor_id: i64,
    code: &str,
    name: &str,
) -> Result<ContractView, String> {
    let code = code.trim();
    let name = name.trim();
    if code.is_empty() || name.is_empty() {
        return Err("Enter a contract code and name.".into());
    }
    let project_id = project_id(conn, project_code)?;
    conn.execute(
        "INSERT INTO contract (project_id, contractor_id, code, name) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(project_id, code) DO UPDATE SET contractor_id = excluded.contractor_id, name = excluded.name",
        params![project_id, contractor_id, code, name],
    )
    .map_err(|err| err.to_string())?;
    load_contract(conn, project_code, code)
}

pub fn save_work_order(conn: &Connection, contract_id: i64, code: &str, name: &str) -> Result<(), String> {
    let code = code.trim();
    let name = name.trim();
    if code.is_empty() || name.is_empty() {
        return Err("Enter a work order code and name.".into());
    }
    conn.execute(
        "INSERT INTO work_order (contract_id, code, name) VALUES (?1, ?2, ?3)
         ON CONFLICT(contract_id, code) DO UPDATE SET name = excluded.name",
        params![contract_id, code, name],
    )
    .map(|_| ())
    .map_err(|err| err.to_string())
}

pub fn add_contract_item(
    conn: &Connection,
    contract_id: i64,
    description: &str,
    quantity: &str,
    rate: &str,
) -> Result<ContractView, String> {
    let description = description.trim();
    if description.is_empty() {
        return Err("Enter a contract bill description.".into());
    }
    let amount = format_number(number(quantity)? * number(rate)?);
    conn.execute(
        "INSERT INTO contract_item (contract_id, description, quantity, rate, amount)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![contract_id, description, quantity.trim(), rate.trim(), amount],
    )
    .map_err(|err| err.to_string())?;
    refresh_value(conn, contract_id)?;
    let (project_code, code): (String, String) = conn
        .query_row(
            "SELECT project.code, contract.code
             FROM contract JOIN project ON project.id = contract.project_id
             WHERE contract.id = ?1",
            params![contract_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|err| err.to_string())?;
    load_contract(conn, &project_code, &code)
}

fn refresh_value(conn: &Connection, contract_id: i64) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT amount FROM contract_item WHERE contract_id = ?1")
        .map_err(|err| err.to_string())?;
    let amounts = stmt
        .query_map(params![contract_id], |row| row.get::<_, String>(0))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let total: f64 = amounts.iter().map(|amount| number(amount)).collect::<Result<Vec<_>, _>>()?.into_iter().sum();
    conn.execute(
        "UPDATE contract SET value = ?1 WHERE id = ?2",
        params![format_number(total), contract_id],
    )
    .map(|_| ())
    .map_err(|err| err.to_string())
}

fn load_contract(conn: &Connection, project_code: &str, code: &str) -> Result<ContractView, String> {
    conn.query_row(
        "SELECT contract.id, contract.code, contract.name, contractor.name, contract.value
         FROM contract
         JOIN project ON project.id = contract.project_id
         JOIN contractor ON contractor.id = contract.contractor_id
         WHERE project.code = ?1 AND contract.code = ?2",
        params![project_code.trim(), code],
        |row| {
            Ok(ContractView {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                contractor_name: row.get(3)?,
                value: row.get(4)?,
            })
        },
    )
    .map_err(|_| format!("Contract {code} was not found."))
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
        return Err("Enter quantity and rate.".into());
    }
    raw.parse().map_err(|_| format!("{raw} is not a number."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_firm;
    use crate::project::save_project;

    #[test]
    fn contract_value_equals_the_contract_boq() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        save_work_order(&firm.conn, contract.id, "WO-1", "Ground floor").unwrap();
        add_contract_item(&firm.conn, contract.id, "Slab", "2.5", "10").unwrap();
        let updated = add_contract_item(&firm.conn, contract.id, "Beam", "1", "20").unwrap();
        assert_eq!(updated.value, "45");
    }
}

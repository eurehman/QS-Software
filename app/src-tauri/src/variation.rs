use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::format_number;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariationResult {
    pub id: i64,
    pub status: String,
    pub revised_sum: String,
}

pub fn save_variation(
    conn: &Connection,
    contract_id: i64,
    kind: &str,
    description: &str,
    amount: &str,
) -> Result<VariationResult, String> {
    let kind = kind.trim();
    if kind != "add" && kind != "omit" && kind != "substitute" {
        return Err("Choose add, omit, or substitute.".into());
    }
    let description = description.trim();
    if description.is_empty() {
        return Err("Enter a variation description.".into());
    }
    let signed = signed_amount(kind, amount)?;
    conn.execute(
        "INSERT INTO variation (contract_id, kind, description, amount, status)
         VALUES (?1, ?2, ?3, ?4, 'draft')",
        params![contract_id, kind, description, format_number(signed)],
    )
    .map_err(|err| err.to_string())?;
    let id = conn.last_insert_rowid();
    result(conn, id)
}

pub fn approve_variation(conn: &Connection, variation_id: i64) -> Result<VariationResult, String> {
    let changed = conn
        .execute(
            "UPDATE variation SET status = 'approved' WHERE id = ?1 AND status = 'draft'",
            params![variation_id],
        )
        .map_err(|err| err.to_string())?;
    if changed == 0 {
        return Err("That variation is not waiting for approval.".into());
    }
    result(conn, variation_id)
}

fn result(conn: &Connection, variation_id: i64) -> Result<VariationResult, String> {
    let (status, contract_id): (String, i64) = conn
        .query_row(
            "SELECT status, contract_id FROM variation WHERE id = ?1",
            params![variation_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Variation was not found.".to_string())?;
    Ok(VariationResult {
        id: variation_id,
        status,
        revised_sum: revised_sum(conn, contract_id)?,
    })
}

fn revised_sum(conn: &Connection, contract_id: i64) -> Result<String, String> {
    let base: String = conn
        .query_row("SELECT value FROM contract WHERE id = ?1", params![contract_id], |row| row.get(0))
        .map_err(|_| "Contract was not found.".to_string())?;
    let mut stmt = conn
        .prepare("SELECT amount FROM variation WHERE contract_id = ?1 AND status = 'approved'")
        .map_err(|err| err.to_string())?;
    let amounts = stmt
        .query_map(params![contract_id], |row| row.get::<_, String>(0))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let adjustment: f64 = amounts.iter().map(|amount| number(amount)).collect::<Result<Vec<_>, _>>()?.into_iter().sum();
    Ok(format_number(number(&base)? + adjustment))
}

fn signed_amount(kind: &str, raw: &str) -> Result<f64, String> {
    let value = number(raw)?;
    match kind {
        "add" => Ok(value.abs()),
        "omit" => Ok(-value.abs()),
        _ => Ok(value),
    }
}

fn number(raw: &str) -> Result<f64, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Enter an amount.".into());
    }
    raw.parse().map_err(|_| format!("{raw} is not a number."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{add_contract_item, save_contract, save_contractor};
    use crate::db::create_firm;
    use crate::project::save_project;

    #[test]
    fn approved_variation_changes_the_sum() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        add_contract_item(&firm.conn, contract.id, "Structure", "10", "100").unwrap();
        let draft = save_variation(&firm.conn, contract.id, "add", "Extra slab", "200").unwrap();
        assert_eq!(draft.revised_sum, "1000");
        let approved = approve_variation(&firm.conn, draft.id).unwrap();
        assert_eq!(approved.revised_sum, "1200");
        let omitted = save_variation(&firm.conn, contract.id, "omit", "Beam", "50").unwrap();
        approve_variation(&firm.conn, omitted.id).unwrap();
        let substituted = save_variation(&firm.conn, contract.id, "substitute", "Finish", "80").unwrap();
        let revised = approve_variation(&firm.conn, substituted.id).unwrap();
        assert_eq!(revised.revised_sum, "1230");
    }
}

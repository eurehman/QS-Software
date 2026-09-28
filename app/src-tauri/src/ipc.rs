use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::format_number;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub id: i64,
    pub certificate_no: i64,
    pub previous: String,
    pub this_bill: String,
    pub retention: String,
    pub advance_recovery: String,
    pub deductions: String,
    pub net_payable: String,
}

pub fn save_certificate(
    conn: &Connection,
    contract_id: i64,
    certificate_no: i64,
    previous: &str,
    work_to_date: &str,
    retention_percent: &str,
    advance_recovery: &str,
    deductions: &str,
) -> Result<Certificate, String> {
    if certificate_no < 1 {
        return Err("Enter a certificate number.".into());
    }
    let previous_amount = amount(previous)?;
    let this_bill = amount(work_to_date)? - previous_amount;
    let retention = this_bill * amount(retention_percent)? / 100.0;
    let advance = amount(advance_recovery)?;
    let other = amount(deductions)?;
    let net = this_bill - retention - advance - other;
    let previous_text = format_number(previous_amount);
    let this_text = format_number(this_bill);
    let retention_text = format_number(retention);
    let advance_text = format_number(advance);
    let deductions_text = format_number(other);
    let net_text = format_number(net);
    conn.execute(
        "INSERT INTO ipc (
            contract_id, certificate_no, previous, this_bill, retention, advance_recovery, deductions, net_payable
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(contract_id, certificate_no) DO UPDATE SET
            previous = excluded.previous,
            this_bill = excluded.this_bill,
            retention = excluded.retention,
            advance_recovery = excluded.advance_recovery,
            deductions = excluded.deductions,
            net_payable = excluded.net_payable,
            status = 'draft'",
        params![
            contract_id,
            certificate_no,
            previous_text,
            this_text,
            retention_text,
            advance_text,
            deductions_text,
            net_text
        ],
    )
    .map_err(|err| err.to_string())?;
    conn.query_row(
        "SELECT id, certificate_no, previous, this_bill, retention, advance_recovery, deductions, net_payable
         FROM ipc WHERE contract_id = ?1 AND certificate_no = ?2",
        params![contract_id, certificate_no],
        |row| {
            Ok(Certificate {
                id: row.get(0)?,
                certificate_no: row.get(1)?,
                previous: row.get(2)?,
                this_bill: row.get(3)?,
                retention: row.get(4)?,
                advance_recovery: row.get(5)?,
                deductions: row.get(6)?,
                net_payable: row.get(7)?,
            })
        },
    )
    .map_err(|err| err.to_string())
}

fn amount(raw: &str) -> Result<f64, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(0.0);
    }
    raw.parse().map_err(|_| format!("{raw} is not a number."))
}

pub fn recommend_bill(conn: &Connection, certificate_id: i64) -> Result<(), String> {
    move_bill(conn, certificate_id, &["draft"], "recommended")
}

pub fn approve_bill(conn: &Connection, certificate_id: i64) -> Result<(), String> {
    move_bill(conn, certificate_id, &["recommended"], "approved")
}

pub fn reject_bill(conn: &Connection, certificate_id: i64) -> Result<(), String> {
    move_bill(conn, certificate_id, &["draft", "recommended"], "rejected")
}

pub fn certify_bill(conn: &Connection, certificate_id: i64) -> Result<(), String> {
    move_bill(conn, certificate_id, &["approved"], "certified")
}

pub fn certify_in_order(conn: &Connection, certificate_id: i64) -> Result<(), String> {
    recommend_bill(conn, certificate_id)?;
    approve_bill(conn, certificate_id)?;
    certify_bill(conn, certificate_id)
}

fn move_bill(conn: &Connection, certificate_id: i64, allowed: &[&str], next: &str) -> Result<(), String> {
    let status: String = conn
        .query_row("SELECT status FROM ipc WHERE id = ?1", params![certificate_id], |row| row.get(0))
        .map_err(|_| "Certificate was not found.".to_string())?;
    if !allowed.iter().any(|item| *item == status) {
        if next == "certified" {
            return Err("An unapproved bill cannot be certified.".into());
        }
        return Err(format!("This bill is {status} and cannot become {next}."));
    }
    conn.execute("UPDATE ipc SET status = ?1 WHERE id = ?2", params![next, certificate_id])
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{add_contract_item, save_contract, save_contractor};
    use crate::db::create_firm;
    use crate::project::save_project;

    #[test]
    fn net_payable_matches_a_worked_certificate() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        add_contract_item(&firm.conn, contract.id, "Structure", "100", "10").unwrap();
        let certificate = save_certificate(&firm.conn, contract.id, 2, "400", "1000", "10", "50", "10").unwrap();
        assert_eq!(certificate.this_bill, "600");
        assert_eq!(certificate.retention, "60");
        assert_eq!(certificate.net_payable, "480");
    }

    #[test]
    fn unapproved_bill_cannot_certify() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        let certificate = save_certificate(&firm.conn, contract.id, 1, "0", "400", "0", "0", "0").unwrap();
        let error = certify_bill(&firm.conn, certificate.id).unwrap_err();
        assert!(error.contains("unapproved"));
        certify_in_order(&firm.conn, certificate.id).unwrap();
    }
}

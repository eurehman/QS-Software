use std::path::Path;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::format_number;
use crate::cost::{self, certified_amount};
use crate::xlsx;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaBuildup {
    pub saleable: String,
    pub common: String,
    pub gfa: String,
    pub schedule: String,
    pub cost_per_sqft: String,
    pub margin: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioView {
    pub live_budget: String,
    pub scenario_budget: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalAccount {
    pub amount: String,
    pub certificate_no: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTotal {
    pub code: String,
    pub total: String,
}

pub fn save_area(conn: &Connection, project_code: &str, name: &str, kind: &str, area: &str) -> Result<AreaBuildup, String> {
    let name = name.trim();
    let kind = kind.trim();
    if name.is_empty() {
        return Err("Enter an area name.".into());
    }
    if kind != "saleable" && kind != "common" {
        return Err("Choose saleable or common.".into());
    }
    let area = number(area)?;
    let project_id = cost::project_id(conn, project_code)?;
    conn.execute(
        "INSERT INTO development_area (project_id, name, kind, area) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, name, kind, format_number(area)],
    )
    .map_err(|err| err.to_string())?;
    area_buildup(conn, project_code)
}

pub fn save_sale_rate(conn: &Connection, project_code: &str, rate: &str) -> Result<AreaBuildup, String> {
    let rate = number(rate)?;
    let project_id = cost::project_id(conn, project_code)?;
    conn.execute(
        "INSERT INTO development_sale (project_id, rate_per_area) VALUES (?1, ?2)
         ON CONFLICT(project_id) DO UPDATE SET rate_per_area = excluded.rate_per_area",
        params![project_id, format_number(rate)],
    )
    .map_err(|err| err.to_string())?;
    area_buildup(conn, project_code)
}

pub fn save_scenario(conn: &Connection, project_code: &str, name: &str, budget: &str) -> Result<ScenarioView, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Enter a scenario name.".into());
    }
    let budget = number(budget)?;
    let project_id = cost::project_id(conn, project_code)?;
    conn.execute(
        "INSERT INTO scenario (project_id, name, budget) VALUES (?1, ?2, ?3)",
        params![project_id, name, format_number(budget)],
    )
    .map_err(|err| err.to_string())?;
    let live = live_budget(conn, project_id)?;
    Ok(ScenarioView { live_budget: live, scenario_budget: format_number(budget) })
}

pub fn close_account(conn: &Connection, project_code: &str) -> Result<FinalAccount, String> {
    let project_id = cost::project_id(conn, project_code)?;
    let (certificate_id, certificate_no, previous, this_bill): (i64, i64, String, String) = conn
        .query_row(
            "SELECT ipc.id, ipc.certificate_no, ipc.previous, ipc.this_bill
             FROM ipc
             JOIN contract ON contract.id = ipc.contract_id
             WHERE contract.project_id = ?1 AND ipc.status = 'certified'
             ORDER BY ipc.certificate_no DESC
             LIMIT 1",
            params![project_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| "Certify a bill before the final account.".to_string())?;
    let amount = format_number(number(&previous)? + number(&this_bill)?);
    conn.execute(
        "INSERT INTO final_account (project_id, certificate_id, amount) VALUES (?1, ?2, ?3)
         ON CONFLICT(project_id) DO UPDATE SET certificate_id = excluded.certificate_id, amount = excluded.amount",
        params![project_id, certificate_id, amount],
    )
    .map_err(|err| err.to_string())?;
    conn.execute("DELETE FROM historical_cost WHERE project_id = ?1 AND code = 'FINAL'", params![project_id])
        .map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO historical_cost (project_id, code, rate, amount) VALUES (?1, 'FINAL', ?2, ?3)",
        params![project_id, amount, amount],
    )
    .map_err(|err| err.to_string())?;
    Ok(FinalAccount { amount, certificate_no })
}

pub fn project_totals(conn: &Connection) -> Result<Vec<ProjectTotal>, String> {
    let mut stmt = conn.prepare("SELECT id, code FROM project ORDER BY code").map_err(|err| err.to_string())?;
    let projects = stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    drop(stmt);
    projects
        .into_iter()
        .map(|(id, code)| {
            let total = certified_amount(conn, id, "this_bill")?;
            Ok(ProjectTotal { code, total: format_number(total) })
        })
        .collect()
}

pub fn integrations_off(conn: &Connection) -> Result<bool, String> {
    let enabled: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(enabled), 0) FROM integration_setting",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(enabled == 0)
}

pub fn set_integration(conn: &Connection, key: &str, enabled: bool) -> Result<bool, String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Enter an integration name.".into());
    }
    conn.execute(
        "INSERT INTO integration_setting (key, enabled) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET enabled = excluded.enabled",
        params![key, if enabled { 1 } else { 0 }],
    )
    .map_err(|err| err.to_string())?;
    integrations_off(conn)
}

pub fn prepare_sheet_exchange(source: &str) -> Result<(), String> {
    let lower = source.trim().to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.contains("docs.google.com") {
        return Err("Use an exported workbook. A Google account is not required.".into());
    }
    Ok(())
}

pub fn export_local_workbook(conn: &Connection, project_code: &str, path: &Path) -> Result<(), String> {
    prepare_sheet_exchange(&path.to_string_lossy())?;
    let _project_id = cost::project_id(conn, project_code)?;
    let rows = vec![
        vec!["QS".to_string(), "local workbook".to_string()],
        vec!["Project".to_string(), project_code.trim().to_string()],
    ];
    xlsx::write_grid(path, &rows)
}

fn area_buildup(conn: &Connection, project_code: &str) -> Result<AreaBuildup, String> {
    let project_id = cost::project_id(conn, project_code)?;
    let mut stmt = conn
        .prepare("SELECT kind, area FROM development_area WHERE project_id = ?1")
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![project_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let mut saleable = 0.0;
    let mut common = 0.0;
    for (kind, area) in rows {
        let area = number(&area)?;
        if kind == "saleable" {
            saleable += area;
        } else {
            common += area;
        }
    }
    let gfa = saleable + common;
    let actual = certified_amount(conn, project_id, "this_bill")?;
    let rate: String = conn
        .query_row(
            "SELECT rate_per_area FROM development_sale WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "0".to_string());
    let revenue = saleable * number(&rate)?;
    let cost_per_sqft = if saleable == 0.0 { 0.0 } else { actual / saleable };
    Ok(AreaBuildup {
        saleable: format_number(saleable),
        common: format_number(common),
        gfa: format_number(gfa),
        schedule: format_number(saleable),
        cost_per_sqft: format_number(cost_per_sqft),
        margin: format_number(revenue - actual),
    })
}

fn live_budget(conn: &Connection, project_id: i64) -> Result<String, String> {
    Ok(conn
        .query_row("SELECT amount FROM cost_budget WHERE project_id = ?1", params![project_id], |row| row.get(0))
        .unwrap_or_else(|_| "0".to_string()))
}

fn number(raw: &str) -> Result<f64, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Enter a number.".into());
    }
    raw.parse().map_err(|_| format!("{raw} is not a number."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{save_contract, save_contractor};
    use crate::cost::save_budget;
    use crate::db::create_firm;
    use crate::ipc::{certify_in_order, save_certificate};
    use crate::project::save_project;
    use crate::xlsx::read_grid;

    #[test]
    fn area_buildup_matches_the_schedule() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        save_area(&firm.conn, "TWR", "Unit 1", "saleable", "500").unwrap();
        save_area(&firm.conn, "TWR", "Unit 2", "saleable", "300").unwrap();
        let buildup = save_area(&firm.conn, "TWR", "Lobby", "common", "200").unwrap();
        assert_eq!(buildup.saleable, "800");
        assert_eq!(buildup.schedule, buildup.saleable);
        assert_eq!(buildup.gfa, "1000");
    }

    #[test]
    fn scenario_does_not_change_the_live_budget() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        save_budget(&firm.conn, "TWR", "2000").unwrap();
        let scenario = save_scenario(&firm.conn, "TWR", "Higher sale", "2500").unwrap();
        assert_eq!(scenario.scenario_budget, "2500");
        assert_eq!(scenario.live_budget, "2000");
    }

    #[test]
    fn final_account_ties_to_the_last_ipc() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        let first = save_certificate(&firm.conn, contract.id, 1, "0", "400", "0", "0", "0").unwrap();
        let second = save_certificate(&firm.conn, contract.id, 2, "400", "1000", "0", "0", "0").unwrap();
        certify_in_order(&firm.conn, first.id).unwrap();
        certify_in_order(&firm.conn, second.id).unwrap();
        let account = close_account(&firm.conn, "TWR").unwrap();
        assert_eq!(account.certificate_no, 2);
        assert_eq!(account.amount, "1000");
        let stored: String = firm
            .conn
            .query_row("SELECT amount FROM historical_cost WHERE code = 'FINAL'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(stored, account.amount);
    }

    #[test]
    fn project_totals_stay_isolated() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "AAA", "First").unwrap();
        save_project(&firm.conn, "BBB", "Second").unwrap();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let first = save_contract(&firm.conn, "AAA", contractor, "C-01", "First package").unwrap();
        let second = save_contract(&firm.conn, "BBB", contractor, "C-02", "Second package").unwrap();
        let bill_a = save_certificate(&firm.conn, first.id, 1, "0", "400", "0", "0", "0").unwrap();
        let bill_b = save_certificate(&firm.conn, second.id, 1, "0", "100", "0", "0", "0").unwrap();
        certify_in_order(&firm.conn, bill_a.id).unwrap();
        certify_in_order(&firm.conn, bill_b.id).unwrap();
        let totals = project_totals(&firm.conn).unwrap();
        let aaa = totals.iter().find(|item| item.code == "AAA").unwrap();
        let bbb = totals.iter().find(|item| item.code == "BBB").unwrap();
        assert_eq!(aaa.total, "400");
        assert_eq!(bbb.total, "100");
    }

    #[test]
    fn app_works_with_integrations_off() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        assert!(integrations_off(&firm.conn).unwrap());
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let still_off = set_integration(&firm.conn, "accounts", false).unwrap();
        assert!(still_off);
        let count: i64 = firm.conn.query_row("SELECT COUNT(*) FROM project", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn core_runs_with_no_network() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let error = prepare_sheet_exchange("https://docs.google.com/spreadsheets/d/abc").unwrap_err();
        assert!(error.contains("Google account"));
        let path = dir.path().join("QS.xlsx");
        export_local_workbook(&firm.conn, "TWR", &path).unwrap();
        let rows = read_grid(&path).unwrap();
        assert_eq!(rows[0][0], "QS");
        assert_eq!(rows[1][1], "TWR");
    }
}

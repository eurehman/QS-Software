use std::collections::BTreeMap;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::codes::{self, format_number};
use crate::cost::{self, certified_amount};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exposure {
    pub actual: String,
    pub open_commitment: String,
    pub total: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorTotal {
    pub vendor: String,
    pub total: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub lines: Vec<VendorTotal>,
    pub selected_vendor: String,
    pub selected_total: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialCheck {
    pub theoretical: String,
    pub actual: String,
    pub wastage: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Forecast {
    pub actual: String,
    pub remaining: String,
    pub eac: String,
    pub cash_flow: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationRollup {
    pub allocated: String,
    pub project_total: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Kpi {
    pub budget: String,
    pub actual: String,
    pub commitment: String,
    pub forecast: String,
    pub paid: String,
}

pub fn save_commitment(
    conn: &Connection,
    project_code: &str,
    description: &str,
    order_amount: &str,
    already_certified: &str,
) -> Result<Exposure, String> {
    let description = description.trim();
    if description.is_empty() {
        return Err("Enter a commitment description.".into());
    }
    let order = number(order_amount)?;
    let recognised = number(already_certified)?;
    if recognised > order {
        return Err("Certified amount cannot exceed the order.".into());
    }
    let project_id = cost::project_id(conn, project_code)?;
    conn.execute(
        "INSERT INTO commitment (project_id, description, order_amount, already_certified)
         VALUES (?1, ?2, ?3, ?4)",
        params![project_id, description, format_number(order), format_number(recognised)],
    )
    .map_err(|err| err.to_string())?;
    exposure(conn, project_id)
}

pub fn add_quote_line(conn: &Connection, project_code: &str, vendor: &str, description: &str, amount: &str) -> Result<(), String> {
    let vendor = vendor.trim();
    let description = description.trim();
    if vendor.is_empty() || description.is_empty() {
        return Err("Enter a vendor and a description.".into());
    }
    let price = number(amount)?;
    let project_id = cost::project_id(conn, project_code)?;
    let rfq_id = ensure_rfq(conn, project_id)?;
    let quotation_id = ensure_quotation(conn, rfq_id, vendor)?;
    conn.execute(
        "INSERT INTO quotation_line (quotation_id, description, amount) VALUES (?1, ?2, ?3)",
        params![quotation_id, description, format_number(price)],
    )
    .map(|_| ())
    .map_err(|err| err.to_string())
}

pub fn comparative_statement(conn: &Connection, project_code: &str) -> Result<Comparison, String> {
    let project_id = cost::project_id(conn, project_code)?;
    let mut stmt = conn
        .prepare(
            "SELECT quotation.vendor, quotation_line.amount
             FROM quotation_line
             JOIN quotation ON quotation.id = quotation_line.quotation_id
             JOIN rfq ON rfq.id = quotation.rfq_id
             JOIN requisition ON requisition.id = rfq.requisition_id
             WHERE requisition.project_id = ?1",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![project_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    if rows.is_empty() {
        return Err("Add quotations before comparing them.".into());
    }
    let mut totals = BTreeMap::<String, f64>::new();
    for (vendor, amount) in rows {
        *totals.entry(vendor).or_insert(0.0) += number(&amount)?;
    }
    let mut lines: Vec<VendorTotal> = totals
        .iter()
        .map(|(vendor, total)| VendorTotal { vendor: vendor.clone(), total: format_number(*total) })
        .collect();
    lines.sort_by(|left, right| left.vendor.cmp(&right.vendor));
    let selected = totals
        .iter()
        .min_by(|left, right| left.1.partial_cmp(right.1).unwrap_or(std::cmp::Ordering::Equal).then(left.0.cmp(right.0)))
        .map(|(vendor, total)| (vendor.clone(), *total))
        .ok_or_else(|| "Add quotations before comparing them.".to_string())?;
    Ok(Comparison {
        lines,
        selected_vendor: selected.0,
        selected_total: format_number(selected.1),
    })
}

pub fn place_order(conn: &Connection, project_code: &str) -> Result<Comparison, String> {
    let comparison = comparative_statement(conn, project_code)?;
    let project_id = cost::project_id(conn, project_code)?;
    let quotation_id: i64 = conn
        .query_row(
            "SELECT quotation.id
             FROM quotation
             JOIN rfq ON rfq.id = quotation.rfq_id
             JOIN requisition ON requisition.id = rfq.requisition_id
             WHERE requisition.project_id = ?1 AND quotation.vendor = ?2",
            params![project_id, comparison.selected_vendor],
            |row| row.get(0),
        )
        .map_err(|_| "The selected quotation was not found.".to_string())?;
    conn.execute(
        "INSERT INTO purchase_order (quotation_id, total) VALUES (?1, ?2)",
        params![quotation_id, comparison.selected_total],
    )
    .map_err(|err| err.to_string())?;
    Ok(comparison)
}

pub fn save_material(conn: &Connection, project_code: &str, name: &str, theoretical: &str) -> Result<i64, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Enter a material name.".into());
    }
    let theoretical = number(theoretical)?;
    let project_id = cost::project_id(conn, project_code)?;
    conn.execute(
        "INSERT INTO material_item (project_id, name, theoretical) VALUES (?1, ?2, ?3)",
        params![project_id, name, format_number(theoretical)],
    )
    .map_err(|err| err.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn add_material_move(conn: &Connection, material_id: i64, kind: &str, quantity: &str) -> Result<MaterialCheck, String> {
    let kind = kind.trim();
    if kind != "receipt" && kind != "issue" {
        return Err("Choose receipt or issue.".into());
    }
    let quantity = number(quantity)?;
    conn.execute(
        "INSERT INTO material_move (material_id, kind, quantity) VALUES (?1, ?2, ?3)",
        params![material_id, kind, format_number(quantity)],
    )
    .map_err(|err| err.to_string())?;
    material_check(conn, material_id)
}

pub fn save_forecast(conn: &Connection, project_code: &str, remaining: &str, cash_flow: &str) -> Result<Forecast, String> {
    let remaining_amount = number(remaining)?;
    let cash = number(cash_flow)?;
    let project_id = cost::project_id(conn, project_code)?;
    conn.execute(
        "INSERT INTO forecast (project_id, remaining, cash_flow) VALUES (?1, ?2, ?3)
         ON CONFLICT(project_id) DO UPDATE SET remaining = excluded.remaining, cash_flow = excluded.cash_flow",
        params![project_id, format_number(remaining_amount), format_number(cash)],
    )
    .map_err(|err| err.to_string())?;
    let actual = certified_amount(conn, project_id, "this_bill")?;
    Ok(Forecast {
        actual: format_number(actual),
        remaining: format_number(remaining_amount),
        eac: format_number(actual + remaining_amount),
        cash_flow: format_number(cash),
    })
}

pub fn save_location_cost(conn: &Connection, location_id: i64, amount: &str) -> Result<(), String> {
    let amount = number(amount)?;
    conn.execute(
        "INSERT INTO location_cost (location_id, amount) VALUES (?1, ?2)
         ON CONFLICT(location_id) DO UPDATE SET amount = excluded.amount",
        params![location_id, format_number(amount)],
    )
    .map(|_| ())
    .map_err(|err| err.to_string())
}

pub fn location_rollup(conn: &Connection, project_code: &str) -> Result<LocationRollup, String> {
    let project_id = cost::project_id(conn, project_code)?;
    let mut stmt = conn
        .prepare(
            "SELECT location_cost.amount
             FROM location_cost
             JOIN location_node ON location_node.id = location_cost.location_id
             WHERE location_node.project_id = ?1",
        )
        .map_err(|err| err.to_string())?;
    let amounts = stmt
        .query_map(params![project_id], |row| row.get::<_, String>(0))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let allocated: f64 = amounts.iter().map(|amount| number(amount)).sum::<Result<f64, _>>()?;
    let project_total = certified_amount(conn, project_id, "this_bill")?;
    Ok(LocationRollup {
        allocated: format_number(allocated),
        project_total: format_number(project_total),
    })
}

pub fn save_library_rate(conn: &Connection, kind: &str, code: &str, rate: &str) -> Result<i64, String> {
    let kind = kind.trim();
    if kind != "historical" && kind != "market" {
        return Err("Choose a historical or market rate.".into());
    }
    let code = code.trim();
    if code.is_empty() {
        return Err("Enter a rate code.".into());
    }
    let rate = number(rate)?;
    conn.execute(
        "INSERT INTO rate_library (kind, code, rate) VALUES (?1, ?2, ?3)
         ON CONFLICT(kind, code) DO UPDATE SET rate = excluded.rate",
        params![kind, code, format_number(rate)],
    )
    .map_err(|err| err.to_string())?;
    conn.query_row(
        "SELECT id FROM rate_library WHERE kind = ?1 AND code = ?2",
        params![kind, code],
        |row| row.get(0),
    )
    .map_err(|err| err.to_string())
}

pub fn apply_library_rate(conn: &Connection, item_id: i64, library_id: i64) -> Result<String, String> {
    let rate: String = conn
        .query_row("SELECT rate FROM rate_library WHERE id = ?1", params![library_id], |row| row.get(0))
        .map_err(|_| "That library rate was not found.".to_string())?;
    codes::set_item_rate(conn, item_id, &rate)?;
    conn.query_row("SELECT amount FROM work_item WHERE id = ?1", params![item_id], |row| row.get(0))
        .map_err(|err| err.to_string())
}

pub fn project_kpi(conn: &Connection, project_code: &str) -> Result<Kpi, String> {
    let project_id = cost::project_id(conn, project_code)?;
    let budget: String = conn
        .query_row("SELECT amount FROM cost_budget WHERE project_id = ?1", params![project_id], |row| row.get(0))
        .unwrap_or_else(|_| "0".to_string());
    let actual = certified_amount(conn, project_id, "this_bill")?;
    let open = open_commitment(conn, project_id)?;
    let remaining: String = conn
        .query_row("SELECT remaining FROM forecast WHERE project_id = ?1", params![project_id], |row| row.get(0))
        .unwrap_or_else(|_| "0".to_string());
    let paid = certified_amount(conn, project_id, "net_payable")?;
    Ok(Kpi {
        budget,
        actual: format_number(actual),
        commitment: format_number(open),
        forecast: format_number(actual + number(&remaining)?),
        paid: format_number(paid),
    })
}

pub fn report_lines(conn: &Connection, project_code: &str) -> Result<Vec<(String, String, String)>, String> {
    let project_id = match cost::project_id(conn, project_code) {
        Ok(id) => id,
        Err(_) => return Ok(Vec::new()),
    };
    let mut lines = Vec::new();
    let commitments: i64 = conn
        .query_row("SELECT COUNT(*) FROM commitment WHERE project_id = ?1", params![project_id], |row| row.get(0))
        .map_err(|err| err.to_string())?;
    if commitments > 0 {
        let position = exposure(conn, project_id)?;
        lines.push(("Commitment".into(), "Open commitment".into(), position.open_commitment));
        lines.push(("Commitment".into(), "Actual plus open commitment".into(), position.total));
    }
    let quotes: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM quotation_line
             JOIN quotation ON quotation.id = quotation_line.quotation_id
             JOIN rfq ON rfq.id = quotation.rfq_id
             JOIN requisition ON requisition.id = rfq.requisition_id
             WHERE requisition.project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if quotes > 0 {
        let comparison = comparative_statement(conn, project_code)?;
        for line in comparison.lines {
            lines.push(("Procurement".into(), line.vendor, line.total));
        }
        lines.push(("Procurement".into(), "Selected quotation".into(), comparison.selected_total));
    }
    let mut materials = conn
        .prepare("SELECT id, name FROM material_item WHERE project_id = ?1 ORDER BY id")
        .map_err(|err| err.to_string())?;
    let material_rows = materials
        .query_map(params![project_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    drop(materials);
    for (id, name) in material_rows {
        let check = material_check(conn, id)?;
        lines.push(("Material".into(), format!("{name} theoretical"), check.theoretical));
        lines.push(("Material".into(), format!("{name} actual"), check.actual));
    }
    if let Ok(forecast) = conn.query_row(
        "SELECT remaining FROM forecast WHERE project_id = ?1",
        params![project_id],
        |row| row.get::<_, String>(0),
    ) {
        let actual = certified_amount(conn, project_id, "this_bill")?;
        lines.push(("Forecast".into(), "Estimate at completion".into(), format_number(actual + number(&forecast)?)));
    }
    let locations: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM location_cost
             JOIN location_node ON location_node.id = location_cost.location_id
             WHERE location_node.project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if locations > 0 {
        let rollup = location_rollup(conn, project_code)?;
        lines.push(("Location".into(), "Allocated".into(), rollup.allocated));
        lines.push(("Location".into(), "Project total".into(), rollup.project_total));
    }
    Ok(lines)
}

fn exposure(conn: &Connection, project_id: i64) -> Result<Exposure, String> {
    let actual = certified_amount(conn, project_id, "this_bill")?;
    let open = open_commitment(conn, project_id)?;
    Ok(Exposure {
        actual: format_number(actual),
        open_commitment: format_number(open),
        total: format_number(actual + open),
    })
}

fn open_commitment(conn: &Connection, project_id: i64) -> Result<f64, String> {
    let mut stmt = conn
        .prepare("SELECT order_amount, already_certified FROM commitment WHERE project_id = ?1")
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![project_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let mut open = 0.0;
    for (order_amount, already) in rows {
        open += number(&order_amount)? - number(&already)?;
    }
    Ok(open)
}

fn material_check(conn: &Connection, material_id: i64) -> Result<MaterialCheck, String> {
    let theoretical: String = conn
        .query_row("SELECT theoretical FROM material_item WHERE id = ?1", params![material_id], |row| row.get(0))
        .map_err(|_| "That material was not found.".to_string())?;
    let mut stmt = conn
        .prepare("SELECT quantity FROM material_move WHERE material_id = ?1 AND kind = 'issue'")
        .map_err(|err| err.to_string())?;
    let issues = stmt
        .query_map(params![material_id], |row| row.get::<_, String>(0))
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let actual: f64 = issues.iter().map(|quantity| number(quantity)).sum::<Result<f64, _>>()?;
    let theoretical_amount = number(&theoretical)?;
    Ok(MaterialCheck {
        theoretical: format_number(theoretical_amount),
        actual: format_number(actual),
        wastage: format_number(actual - theoretical_amount),
    })
}

fn ensure_rfq(conn: &Connection, project_id: i64) -> Result<i64, String> {
    if let Ok(id) = conn.query_row(
        "SELECT rfq.id FROM rfq JOIN requisition ON requisition.id = rfq.requisition_id WHERE requisition.project_id = ?1",
        params![project_id],
        |row| row.get(0),
    ) {
        return Ok(id);
    }
    conn.execute("INSERT INTO requisition (project_id, code) VALUES (?1, 'REQ-1')", params![project_id])
        .map_err(|err| err.to_string())?;
    let requisition_id = conn.last_insert_rowid();
    conn.execute("INSERT INTO rfq (requisition_id, code) VALUES (?1, 'RFQ-1')", params![requisition_id])
        .map_err(|err| err.to_string())?;
    Ok(conn.last_insert_rowid())
}

fn ensure_quotation(conn: &Connection, rfq_id: i64, vendor: &str) -> Result<i64, String> {
    if let Ok(id) = conn.query_row(
        "SELECT id FROM quotation WHERE rfq_id = ?1 AND vendor = ?2",
        params![rfq_id, vendor],
        |row| row.get(0),
    ) {
        return Ok(id);
    }
    conn.execute("INSERT INTO quotation (rfq_id, vendor) VALUES (?1, ?2)", params![rfq_id, vendor])
        .map_err(|err| err.to_string())?;
    Ok(conn.last_insert_rowid())
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
    use crate::codes::{save_code, save_item};
    use crate::contract::{save_contract, save_contractor};
    use crate::db::create_firm;
    use crate::ipc::{certify_in_order, save_certificate};
    use crate::project::{add_location, save_project};

    fn firm_with_project() -> (tempfile::TempDir, crate::db::FirmFile) {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        (dir, firm)
    }

    #[test]
    fn commitment_is_not_counted_twice() {
        let (_dir, firm) = firm_with_project();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        let certificate = save_certificate(&firm.conn, contract.id, 1, "0", "400", "0", "0", "0").unwrap();
        certify_in_order(&firm.conn, certificate.id).unwrap();
        let exposure = save_commitment(&firm.conn, "TWR", "Structure order", "1000", "400").unwrap();
        assert_eq!(exposure.actual, "400");
        assert_eq!(exposure.open_commitment, "600");
        assert_eq!(exposure.total, "1000");
    }

    #[test]
    fn comparative_statement_totals() {
        let (_dir, firm) = firm_with_project();
        add_quote_line(&firm.conn, "TWR", "Alpha", "Cement", "100").unwrap();
        add_quote_line(&firm.conn, "TWR", "Alpha", "Steel", "50").unwrap();
        add_quote_line(&firm.conn, "TWR", "Beta", "Cement", "80").unwrap();
        add_quote_line(&firm.conn, "TWR", "Beta", "Steel", "40").unwrap();
        let statement = comparative_statement(&firm.conn, "TWR").unwrap();
        assert_eq!(statement.lines[0].total, "150");
        assert_eq!(statement.lines[1].total, "120");
        let order = place_order(&firm.conn, "TWR").unwrap();
        assert_eq!(order.selected_vendor, "Beta");
        assert_eq!(order.selected_total, "120");
    }

    #[test]
    fn theoretical_versus_actual() {
        let (_dir, firm) = firm_with_project();
        let material = save_material(&firm.conn, "TWR", "Cement", "100").unwrap();
        add_material_move(&firm.conn, material, "receipt", "120").unwrap();
        let check = add_material_move(&firm.conn, material, "issue", "110").unwrap();
        assert_eq!(check.theoretical, "100");
        assert_eq!(check.actual, "110");
        assert_eq!(check.wastage, "10");
    }

    #[test]
    fn eac_equals_actual_plus_remaining() {
        let (_dir, firm) = firm_with_project();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        let certificate = save_certificate(&firm.conn, contract.id, 1, "0", "1000", "0", "0", "0").unwrap();
        certify_in_order(&firm.conn, certificate.id).unwrap();
        let forecast = save_forecast(&firm.conn, "TWR", "500", "200").unwrap();
        assert_eq!(forecast.eac, "1500");
        assert_eq!(forecast.cash_flow, "200");
    }

    #[test]
    fn location_totals_equal_the_project() {
        let (_dir, firm) = firm_with_project();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        let certificate = save_certificate(&firm.conn, contract.id, 1, "0", "1000", "0", "0", "0").unwrap();
        certify_in_order(&firm.conn, certificate.id).unwrap();
        let tower = add_location(&firm.conn, "TWR", None, "", "Tower", "A").unwrap();
        let floor = add_location(&firm.conn, "TWR", Some(tower.id), "", "Floor", "1").unwrap();
        let unit = add_location(&firm.conn, "TWR", Some(floor.id), "", "Unit", "1").unwrap();
        save_location_cost(&firm.conn, tower.id, "600").unwrap();
        save_location_cost(&firm.conn, floor.id, "250").unwrap();
        save_location_cost(&firm.conn, unit.id, "150").unwrap();
        let rollup = location_rollup(&firm.conn, "TWR").unwrap();
        assert_eq!(rollup.allocated, "1000");
        assert_eq!(rollup.project_total, rollup.allocated);
    }

    #[test]
    fn a_past_rate_can_be_applied() {
        let (_dir, firm) = firm_with_project();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        let item = save_item(&firm.conn, "TWR", None, 1, "Slab", "2", "5", wbs.id, cbs.id, cost.id, unit.id).unwrap();
        let library = save_library_rate(&firm.conn, "historical", "C310", "100").unwrap();
        let amount = apply_library_rate(&firm.conn, item.id, library).unwrap();
        assert_eq!(amount, "200");
    }

    #[test]
    fn kpi_ties_to_the_source() {
        let (_dir, firm) = firm_with_project();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        let certificate = save_certificate(&firm.conn, contract.id, 1, "0", "400", "0", "0", "0").unwrap();
        certify_in_order(&firm.conn, certificate.id).unwrap();
        crate::cost::save_budget(&firm.conn, "TWR", "2000").unwrap();
        save_commitment(&firm.conn, "TWR", "Structure order", "1000", "400").unwrap();
        save_forecast(&firm.conn, "TWR", "500", "200").unwrap();
        let kpi = project_kpi(&firm.conn, "TWR").unwrap();
        assert_eq!(kpi.budget, "2000");
        assert_eq!(kpi.actual, "400");
        assert_eq!(kpi.commitment, "600");
        assert_eq!(kpi.forecast, "900");
        assert_eq!(kpi.paid, "400");
    }
}

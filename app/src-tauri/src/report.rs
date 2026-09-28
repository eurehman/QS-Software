use std::path::Path;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::measure::sheet_total;
use crate::xlsx;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportLine {
    pub section: String,
    pub label: String,
    pub figure: String,
}

pub fn screen_report(conn: &Connection, project_code: &str) -> Result<Vec<ReportLine>, String> {
    let mut lines = Vec::new();
    let mut items = conn
        .prepare(
            "SELECT work_item.id, work_item.name, work_item.amount
             FROM work_item
             JOIN project ON project.id = work_item.project_id
             WHERE project.code = ?1
             ORDER BY work_item.id",
        )
        .map_err(|err| err.to_string())?;
    let rows = items
        .query_map(params![project_code.trim()], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    drop(items);
    for (id, name, amount) in rows {
        lines.push(ReportLine {
            section: "BOQ".into(),
            label: name.clone(),
            figure: amount,
        });
        let measured: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM measure_line WHERE item_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|err| err.to_string())?;
        if measured > 0 {
            lines.push(ReportLine {
                section: "Measurement".into(),
                label: format!("{name} sheet"),
                figure: sheet_total(conn, id)?,
            });
        }
    }
    let mut certificates = conn
        .prepare(
            "SELECT ipc.certificate_no, ipc.this_bill, ipc.net_payable
             FROM ipc
             JOIN contract ON contract.id = ipc.contract_id
             JOIN project ON project.id = contract.project_id
             WHERE project.code = ?1
             ORDER BY ipc.certificate_no",
        )
        .map_err(|err| err.to_string())?;
    let bills = certificates
        .query_map(params![project_code.trim()], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    for (number, this_bill, net_payable) in bills {
        lines.push(ReportLine {
            section: "IPC".into(),
            label: format!("Certificate {number} this bill"),
            figure: this_bill,
        });
        lines.push(ReportLine {
            section: "IPC".into(),
            label: format!("Certificate {number} net payable"),
            figure: net_payable,
        });
    }
    Ok(lines)
}

pub fn export_report(conn: &Connection, project_code: &str, path: &Path) -> Result<Vec<ReportLine>, String> {
    let lines = screen_report(conn, project_code)?;
    let mut rows = vec![vec!["Section".to_string(), "Label".to_string(), "Figure".to_string()]];
    for line in &lines {
        rows.push(vec![line.section.clone(), line.label.clone(), line.figure.clone()]);
    }
    xlsx::write_grid(path, &rows)?;
    let printed = read_printed(path)?;
    if printed != lines {
        return Err("Printed figures do not match the screen.".into());
    }
    Ok(lines)
}

fn read_printed(path: &Path) -> Result<Vec<ReportLine>, String> {
    let rows = xlsx::read_grid(path)?;
    rows.into_iter()
        .skip(1)
        .map(|row| {
            if row.len() < 3 {
                return Err("The printed report is missing a figure.".into());
            }
            Ok(ReportLine {
                section: row[0].clone(),
                label: row[1].clone(),
                figure: row[2].clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::{save_code, save_item};
    use crate::contract::{save_contract, save_contractor};
    use crate::db::create_firm;
    use crate::ipc::save_certificate;
    use crate::measure::add_line;
    use crate::project::save_project;

    #[test]
    fn printed_figures_match_the_screen() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let wbs = save_code(&firm.conn, "wbs", "03.10", "Concrete").unwrap();
        let cbs = save_code(&firm.conn, "cbs", "STR", "Structure").unwrap();
        let cost = save_code(&firm.conn, "cost", "C310", "In-situ concrete").unwrap();
        let unit = save_code(&firm.conn, "unit", "m3", "Cubic metre").unwrap();
        save_item(&firm.conn, "TWR", None, 1, "Preliminaries", "1", "10", wbs.id, cbs.id, cost.id, unit.id).unwrap();
        let slab = save_item(&firm.conn, "TWR", None, 1, "Slab", "0", "10", wbs.id, cbs.id, cost.id, unit.id).unwrap();
        add_line(&firm.conn, slab.id, "Bay A", "2", "2.5", "2", "").unwrap();
        add_line(&firm.conn, slab.id, "Bay B", "1", "5", "", "").unwrap();
        let contractor = save_contractor(&firm.conn, "Alpha Builders").unwrap();
        let contract = save_contract(&firm.conn, "TWR", contractor, "C-01", "Structure package").unwrap();
        save_certificate(&firm.conn, contract.id, 2, "400", "1000", "10", "50", "10").unwrap();
        let screen = screen_report(&firm.conn, "TWR").unwrap();
        let path = dir.path().join("Report.xlsx");
        let printed = export_report(&firm.conn, "TWR", &path).unwrap();
        assert_eq!(printed, screen);
        let figures: Vec<&str> = screen.iter().map(|line| line.figure.as_str()).collect();
        assert_eq!(figures, ["10", "150", "15", "600", "480"]);
    }
}

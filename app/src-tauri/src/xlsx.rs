use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};
use rust_xlsxwriter::Workbook;

pub fn read_grid(path: &Path) -> Result<Vec<Vec<String>>, String> {
    let mut workbook = open_workbook_auto(path).map_err(|err| err.to_string())?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| "The workbook has no sheet.".to_string())?
        .map_err(|err| err.to_string())?;
    let mut rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(cell_text).collect())
        .collect();
    while rows.last().is_some_and(|row| row.iter().all(|cell| cell.is_empty())) {
        rows.pop();
    }
    Ok(rows)
}

pub fn write_grid(path: &Path, rows: &[Vec<String>]) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    for (row_index, row) in rows.iter().enumerate() {
        for (col_index, raw) in row.iter().enumerate() {
            if raw.is_empty() {
                continue;
            }
            if is_plain_number(raw) {
                let number: f64 = raw.parse().map_err(|_| format!("Could not write {raw}"))?;
                sheet
                    .write_number(row_index as u32, col_index as u16, number)
                    .map_err(|err| err.to_string())?;
            } else {
                sheet
                    .write_string(row_index as u32, col_index as u16, raw)
                    .map_err(|err| err.to_string())?;
            }
        }
    }
    workbook.save(path).map_err(|err| err.to_string())
}

fn is_plain_number(raw: &str) -> bool {
    !raw.is_empty()
        && raw.chars().all(|char| char.is_ascii_digit() || char == '.' || char == '-')
        && raw.parse::<f64>().is_ok()
}

fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Float(value) => format_float(*value),
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => if *value { "TRUE" } else { "FALSE" }.into(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        Data::DateTime(value) => value.to_string(),
        Data::Error(value) => format!("{value:?}"),
    }
}

fn format_float(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }
    let text = format!("{value:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workbook_round_trip_keeps_text_numbers_and_formulas() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bill.xlsx");
        let rows = vec![
            vec!["Item".into(), "Qty".into(), "Amount".into()],
            vec!["Concrete".into(), "2.5".into(), "=B2*10".into()],
        ];
        write_grid(&path, &rows).unwrap();
        let read = read_grid(&path).unwrap();
        assert_eq!(read, rows);
    }
}

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::xlsx;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecord {
    pub id: i64,
    pub created_at: String,
    pub folder: String,
    pub created_by: String,
    pub verified: bool,
}

pub fn local_server(conn: &Connection) -> Result<String, String> {
    match conn.query_row(
        "SELECT value FROM app_meta WHERE key = 'local_server'",
        [],
        |row| row.get(0),
    ) {
        Ok(value) => Ok(value),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
        Err(err) => Err(err.to_string()),
    }
}

pub fn set_local_server(conn: &Connection, folder: &str) -> Result<(), String> {
    let folder = folder.trim();
    if folder.is_empty() {
        return Err("Choose a QS Local Server folder.".into());
    }
    let path = PathBuf::from(folder);
    if !path.is_dir() {
        return Err("That QS Local Server folder was not found.".into());
    }
    conn.execute(
        "INSERT INTO app_meta (key, value) VALUES ('local_server', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![path.display().to_string()],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

pub fn create_backup(
    conn: &Connection,
    created_by: &str,
    app_version: &str,
    schema_version: i64,
    firm_path: &str,
) -> Result<BackupRecord, String> {
    let server = local_server(conn)?;
    if server.is_empty() {
        return Err("Set the QS Local Server folder before creating a backup.".into());
    }
    let created_at = utc_stamp();
    let company = Path::new(firm_path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("company");
    let year = &created_at[..4];
    let folder = PathBuf::from(&server)
        .join("QS Software Backups")
        .join(safe_name(company))
        .join(year)
        .join(&created_at);
    fs::create_dir_all(&folder).map_err(|err| err.to_string())?;

    let sheet = sheet_rows(conn)?;
    let info = vec![
        vec!["Field".into(), "Value".into()],
        vec!["Created at".into(), format!("{created_at} UTC")],
        vec!["Created by".into(), created_by.into()],
        vec!["App version".into(), app_version.into()],
        vec!["Schema version".into(), schema_version.to_string()],
        vec!["Firm file".into(), firm_path.into()],
        vec!["Company".into(), company.into()],
        vec!["Files".into(), "Working_Sheet.xlsx".into()],
    ];
    xlsx::write_grid(&folder.join("Working_Sheet.xlsx"), &sheet)?;
    xlsx::write_grid(&folder.join("Backup_Info.xlsx"), &info)?;
    verify_written(&folder, &sheet, &info)?;

    conn.execute(
        "INSERT INTO backup_record (created_at, folder, created_by, verified)
         VALUES (?1, ?2, ?3, 1)",
        params![created_at, folder.display().to_string(), created_by],
    )
    .map_err(|err| err.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(BackupRecord {
        id,
        created_at,
        folder: folder.display().to_string(),
        created_by: created_by.to_string(),
        verified: true,
    })
}

pub fn list_backups(conn: &Connection) -> Result<Vec<BackupRecord>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, created_at, folder, created_by, verified
             FROM backup_record ORDER BY id DESC",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BackupRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                folder: row.get(2)?,
                created_by: row.get(3)?,
                verified: row.get::<_, i64>(4)? == 1,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|err| err.to_string())
}

pub fn restore_backup(conn: &Connection, folder: &str, username: &str) -> Result<(), String> {
    let path = PathBuf::from(folder.trim());
    let sheet_path = path.join("Working_Sheet.xlsx");
    if !sheet_path.is_file() {
        return Err("That backup has no Working_Sheet.xlsx.".into());
    }
    let rows = xlsx::read_grid(&sheet_path)?;
    let mut cells = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        for (col_index, raw) in row.iter().enumerate() {
            if !raw.is_empty() {
                cells.push((row_index as i64, col_index as i64, raw.clone()));
            }
        }
    }
    crate::audit::save_cells(conn, username, &cells)
}

fn sheet_rows(conn: &Connection) -> Result<Vec<Vec<String>>, String> {
    let mut stmt = conn
        .prepare("SELECT sheet_row, sheet_col, raw FROM sheet_cell")
        .map_err(|err| err.to_string())?;
    let cells = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    let mut rows: Vec<Vec<String>> = Vec::new();
    for (row, col, raw) in cells {
        if row < 0 || col < 0 {
            continue;
        }
        let row = row as usize;
        let col = col as usize;
        if rows.len() <= row {
            rows.resize(row + 1, Vec::new());
        }
        if rows[row].len() <= col {
            rows[row].resize(col + 1, String::new());
        }
        rows[row][col] = raw;
    }
    Ok(rows)
}

fn verify_written(folder: &Path, sheet: &[Vec<String>], info: &[Vec<String>]) -> Result<(), String> {
    let read_sheet = xlsx::read_grid(&folder.join("Working_Sheet.xlsx"))?;
    let read_info = xlsx::read_grid(&folder.join("Backup_Info.xlsx"))?;
    if read_sheet != sheet || read_info != info {
        return Err("Backup verification failed. The Excel files do not match the firm data.".into());
    }
    Ok(())
}

fn safe_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|char| if char.is_ascii_alphanumeric() || char == '-' || char == '_' { char } else { '_' })
        .collect();
    if cleaned.is_empty() { "company".into() } else { cleaned }
}

pub fn utc_stamp() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let tod = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = tod / 3600;
    let minute = (tod % 3600) / 60;
    let second = tod % 60;
    format!("{year:04}-{month:02}-{day:02}_{hour:02}{minute:02}{second:02}")
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::create_administrator;
    use crate::db::create_firm;

    #[test]
    fn backup_opens_in_excel_and_restores_the_sheet() {
        let dir = tempfile::tempdir().unwrap();
        let firm_path = dir.path().join("company.qsdb");
        let firm = create_firm(&firm_path, "0.1.0").unwrap();
        create_administrator(&firm.conn, "admin", "password1").unwrap();
        firm.conn
            .execute(
                "INSERT INTO sheet_cell (sheet_row, sheet_col, raw) VALUES (0, 0, 'Concrete'), (0, 1, '2.5'), (0, 2, '=B1*10')",
                [],
            )
            .unwrap();
        let server = dir.path().join("server");
        fs::create_dir(&server).unwrap();
        set_local_server(&firm.conn, server.to_str().unwrap()).unwrap();

        let record = create_backup(
            &firm.conn,
            "admin",
            "0.1.0",
            firm.schema_version,
            firm_path.to_str().unwrap(),
        )
        .unwrap();
        assert!(record.verified);
        let sheet = xlsx::read_grid(&PathBuf::from(&record.folder).join("Working_Sheet.xlsx")).unwrap();
        assert_eq!(
            sheet,
            vec![vec!["Concrete".to_string(), "2.5".to_string(), "=B1*10".to_string()]]
        );
        let info = xlsx::read_grid(&PathBuf::from(&record.folder).join("Backup_Info.xlsx")).unwrap();
        assert_eq!(info[0], vec!["Field".to_string(), "Value".to_string()]);

        firm.conn.execute("DELETE FROM sheet_cell", []).unwrap();
        restore_backup(&firm.conn, &record.folder, "admin").unwrap();
        let restored: String = firm
            .conn
            .query_row("SELECT raw FROM sheet_cell WHERE sheet_row = 0 AND sheet_col = 0", [], |row| row.get(0))
            .unwrap();
        assert_eq!(restored, "Concrete");
        let history = list_backups(&firm.conn).unwrap();
        assert_eq!(history.len(), 1);
        assert!(history[0].folder.contains("QS Software Backups"));
    }
}

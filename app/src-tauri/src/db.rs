use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{params, Connection};

pub const SCHEMA_VERSION: i64 = 19;

#[derive(Debug)]
pub struct FirmFile {
    pub path: PathBuf,
    pub schema_version: i64,
    pub app_version: String,
    pub conn: Connection,
}

pub fn create_firm(path: &Path, app_version: &str) -> Result<FirmFile, String> {
    if path.exists() {
        return Err("A firm file already exists at that path.".into());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    open_connection(path, app_version, true)
}

pub fn open_firm(path: &Path, app_version: &str) -> Result<FirmFile, String> {
    if !path.exists() {
        return Err("That firm file was not found.".into());
    }
    open_connection(path, app_version, false)
}

fn open_connection(path: &Path, app_version: &str, fresh: bool) -> Result<FirmFile, String> {
    let conn = Connection::open(path).map_err(|err| err.to_string())?;
    conn.busy_timeout(Duration::from_secs(5)).map_err(|err| err.to_string())?;
    conn.execute_batch("PRAGMA journal_mode=WAL;").map_err(|err| err.to_string())?;
    let current = schema_version(&conn).map_err(|err| err.to_string())?;
    if !fresh && current > SCHEMA_VERSION {
        return Err("This firm file was written by a newer version of QS.".into());
    }
    if current < SCHEMA_VERSION {
        if current > 0 {
            let backup = path.with_extension(format!("bak-v{current}.qsdb"));
            fs::copy(path, backup).map_err(|err| err.to_string())?;
        }
        migrate(&conn, current).map_err(|err| err.to_string())?;
    }
    conn.execute(
        "INSERT INTO app_meta (key, value) VALUES ('app_version', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![app_version],
    )
    .map_err(|err| err.to_string())?;
    Ok(FirmFile {
        path: path.to_path_buf(),
        schema_version: SCHEMA_VERSION,
        app_version: app_version.to_string(),
        conn,
    })
}

fn schema_version(conn: &Connection) -> rusqlite::Result<i64> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'app_meta'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Ok(0);
    }
    let value: String = conn
        .query_row(
            "SELECT value FROM app_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "0".into());
    Ok(value.parse().unwrap_or(0))
}

fn migrate(conn: &Connection, from: i64) -> rusqlite::Result<()> {
    if from < 1 {
        conn.execute_batch(
            "
            CREATE TABLE app_meta (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );
            INSERT INTO app_meta (key, value) VALUES ('schema_version', '1');
            ",
        )?;
    }
    if from < 2 {
        conn.execute_batch(
            "
            CREATE TABLE user_account (
              id INTEGER PRIMARY KEY,
              username TEXT NOT NULL COLLATE NOCASE UNIQUE,
              display_name TEXT NOT NULL,
              password_hash TEXT NOT NULL,
              is_administrator INTEGER NOT NULL CHECK (is_administrator IN (0, 1))
            );
            CREATE UNIQUE INDEX one_administrator
              ON user_account (is_administrator)
              WHERE is_administrator = 1;
            UPDATE app_meta SET value = '2' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 3 {
        conn.execute_batch(
            "
            CREATE TABLE role (
              id INTEGER PRIMARY KEY,
              name TEXT NOT NULL COLLATE NOCASE UNIQUE
            );
            CREATE TABLE role_permission (
              role_id INTEGER NOT NULL REFERENCES role(id) ON DELETE CASCADE,
              permission TEXT NOT NULL,
              PRIMARY KEY (role_id, permission)
            );
            CREATE TABLE role_module (
              role_id INTEGER NOT NULL REFERENCES role(id) ON DELETE CASCADE,
              module TEXT NOT NULL,
              PRIMARY KEY (role_id, module)
            );
            CREATE TABLE role_project (
              role_id INTEGER NOT NULL REFERENCES role(id) ON DELETE CASCADE,
              project_code TEXT NOT NULL,
              PRIMARY KEY (role_id, project_code)
            );
            ALTER TABLE user_account ADD COLUMN role_id INTEGER REFERENCES role(id);
            UPDATE app_meta SET value = '3' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 4 {
        conn.execute_batch(
            "
            CREATE TABLE sheet_cell (
              sheet_row INTEGER NOT NULL,
              sheet_col INTEGER NOT NULL,
              raw TEXT NOT NULL,
              PRIMARY KEY (sheet_row, sheet_col)
            );
            UPDATE app_meta SET value = '4' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 5 {
        conn.execute_batch(
            "
            CREATE TABLE backup_record (
              id INTEGER PRIMARY KEY,
              created_at TEXT NOT NULL,
              folder TEXT NOT NULL,
              created_by TEXT NOT NULL,
              verified INTEGER NOT NULL CHECK (verified IN (0, 1))
            );
            UPDATE app_meta SET value = '5' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 6 {
        conn.execute_batch(
            "
            CREATE TABLE audit_event (
              id INTEGER PRIMARY KEY,
              created_at TEXT NOT NULL,
              username TEXT NOT NULL,
              action TEXT NOT NULL,
              target TEXT NOT NULL,
              old_value TEXT NOT NULL,
              new_value TEXT NOT NULL
            );
            UPDATE app_meta SET value = '6' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 7 {
        conn.execute_batch(
            "
            CREATE TABLE project (
              id INTEGER PRIMARY KEY,
              code TEXT NOT NULL COLLATE NOCASE UNIQUE,
              name TEXT NOT NULL
            );
            CREATE TABLE location_node (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              parent_id INTEGER REFERENCES location_node(id) ON DELETE CASCADE,
              label TEXT NOT NULL,
              name TEXT NOT NULL
            );
            UPDATE app_meta SET value = '7' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 8 {
        conn.execute_batch(
            "
            CREATE TABLE code_entry (
              id INTEGER PRIMARY KEY,
              kind TEXT NOT NULL CHECK (kind IN ('wbs', 'cbs', 'cost', 'unit')),
              code TEXT NOT NULL COLLATE NOCASE,
              name TEXT NOT NULL,
              UNIQUE (kind, code)
            );
            CREATE TABLE work_item (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              name TEXT NOT NULL,
              wbs_id INTEGER NOT NULL REFERENCES code_entry(id),
              cbs_id INTEGER NOT NULL REFERENCES code_entry(id),
              cost_id INTEGER NOT NULL REFERENCES code_entry(id),
              unit_id INTEGER NOT NULL REFERENCES code_entry(id)
            );
            UPDATE app_meta SET value = '8' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 9 {
        conn.execute_batch(
            "
            ALTER TABLE work_item ADD COLUMN parent_id INTEGER REFERENCES work_item(id);
            ALTER TABLE work_item ADD COLUMN version_no INTEGER NOT NULL DEFAULT 1;
            ALTER TABLE work_item ADD COLUMN quantity TEXT NOT NULL DEFAULT '';
            ALTER TABLE work_item ADD COLUMN rate TEXT NOT NULL DEFAULT '';
            ALTER TABLE work_item ADD COLUMN amount TEXT NOT NULL DEFAULT '';
            UPDATE app_meta SET value = '9' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 10 {
        conn.execute_batch(
            "
            CREATE TABLE measure_line (
              id INTEGER PRIMARY KEY,
              item_id INTEGER NOT NULL REFERENCES work_item(id) ON DELETE CASCADE,
              description TEXT NOT NULL,
              times TEXT NOT NULL DEFAULT '',
              length TEXT NOT NULL DEFAULT '',
              width TEXT NOT NULL DEFAULT '',
              height TEXT NOT NULL DEFAULT ''
            );
            UPDATE app_meta SET value = '10' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 11 {
        conn.execute_batch(
            "
            CREATE TABLE rate_buildup (
              item_id INTEGER PRIMARY KEY REFERENCES work_item(id) ON DELETE CASCADE,
              material_qty TEXT NOT NULL,
              material_rate TEXT NOT NULL,
              labour_qty TEXT NOT NULL,
              labour_rate TEXT NOT NULL,
              plant_qty TEXT NOT NULL,
              plant_rate TEXT NOT NULL,
              wastage_percent TEXT NOT NULL,
              overhead_percent TEXT NOT NULL,
              profit_percent TEXT NOT NULL,
              composite_rate TEXT NOT NULL
            );
            UPDATE app_meta SET value = '11' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 12 {
        conn.execute_batch(
            "
            CREATE TABLE estimate (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              kind TEXT NOT NULL CHECK (kind IN ('boq', 'area')),
              version_no INTEGER NOT NULL,
              area TEXT NOT NULL DEFAULT '',
              rate TEXT NOT NULL DEFAULT '',
              total TEXT NOT NULL
            );
            UPDATE app_meta SET value = '12' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 13 {
        conn.execute_batch(
            "
            CREATE TABLE contractor (
              id INTEGER PRIMARY KEY,
              name TEXT NOT NULL COLLATE NOCASE UNIQUE
            );
            CREATE TABLE contract (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              contractor_id INTEGER NOT NULL REFERENCES contractor(id),
              code TEXT NOT NULL COLLATE NOCASE,
              name TEXT NOT NULL,
              value TEXT NOT NULL DEFAULT '0',
              UNIQUE (project_id, code)
            );
            CREATE TABLE work_order (
              id INTEGER PRIMARY KEY,
              contract_id INTEGER NOT NULL REFERENCES contract(id) ON DELETE CASCADE,
              code TEXT NOT NULL COLLATE NOCASE,
              name TEXT NOT NULL,
              UNIQUE (contract_id, code)
            );
            CREATE TABLE contract_item (
              id INTEGER PRIMARY KEY,
              contract_id INTEGER NOT NULL REFERENCES contract(id) ON DELETE CASCADE,
              description TEXT NOT NULL,
              quantity TEXT NOT NULL,
              rate TEXT NOT NULL,
              amount TEXT NOT NULL
            );
            UPDATE app_meta SET value = '13' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 14 {
        conn.execute_batch(
            "
            CREATE TABLE ipc (
              id INTEGER PRIMARY KEY,
              contract_id INTEGER NOT NULL REFERENCES contract(id) ON DELETE CASCADE,
              certificate_no INTEGER NOT NULL,
              previous TEXT NOT NULL,
              this_bill TEXT NOT NULL,
              retention TEXT NOT NULL,
              advance_recovery TEXT NOT NULL,
              deductions TEXT NOT NULL,
              net_payable TEXT NOT NULL,
              UNIQUE (contract_id, certificate_no)
            );
            UPDATE app_meta SET value = '14' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 15 {
        conn.execute_batch(
            "
            CREATE TABLE variation (
              id INTEGER PRIMARY KEY,
              contract_id INTEGER NOT NULL REFERENCES contract(id) ON DELETE CASCADE,
              kind TEXT NOT NULL CHECK (kind IN ('add', 'omit', 'substitute')),
              description TEXT NOT NULL,
              amount TEXT NOT NULL,
              status TEXT NOT NULL CHECK (status IN ('draft', 'approved'))
            );
            UPDATE app_meta SET value = '15' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 16 {
        conn.execute_batch(
            "
            CREATE TABLE cost_budget (
              project_id INTEGER PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
              amount TEXT NOT NULL
            );
            UPDATE app_meta SET value = '16' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 17 {
        conn.execute_batch(
            "
            CREATE TABLE boq_approval (
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              version_no INTEGER NOT NULL,
              PRIMARY KEY (project_id, version_no)
            );
            UPDATE app_meta SET value = '17' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 18 {
        conn.execute_batch(
            "
            ALTER TABLE ipc ADD COLUMN status TEXT NOT NULL DEFAULT 'draft';
            UPDATE ipc SET status = 'certified';
            CREATE TABLE commitment (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              description TEXT NOT NULL,
              order_amount TEXT NOT NULL,
              already_certified TEXT NOT NULL
            );
            CREATE TABLE requisition (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              code TEXT NOT NULL
            );
            CREATE TABLE rfq (
              id INTEGER PRIMARY KEY,
              requisition_id INTEGER NOT NULL REFERENCES requisition(id) ON DELETE CASCADE,
              code TEXT NOT NULL
            );
            CREATE TABLE quotation (
              id INTEGER PRIMARY KEY,
              rfq_id INTEGER NOT NULL REFERENCES rfq(id) ON DELETE CASCADE,
              vendor TEXT NOT NULL,
              UNIQUE (rfq_id, vendor)
            );
            CREATE TABLE quotation_line (
              id INTEGER PRIMARY KEY,
              quotation_id INTEGER NOT NULL REFERENCES quotation(id) ON DELETE CASCADE,
              description TEXT NOT NULL,
              amount TEXT NOT NULL
            );
            CREATE TABLE purchase_order (
              id INTEGER PRIMARY KEY,
              quotation_id INTEGER NOT NULL REFERENCES quotation(id),
              total TEXT NOT NULL
            );
            CREATE TABLE material_item (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              name TEXT NOT NULL,
              theoretical TEXT NOT NULL
            );
            CREATE TABLE material_move (
              id INTEGER PRIMARY KEY,
              material_id INTEGER NOT NULL REFERENCES material_item(id) ON DELETE CASCADE,
              kind TEXT NOT NULL CHECK (kind IN ('receipt', 'issue')),
              quantity TEXT NOT NULL
            );
            CREATE TABLE forecast (
              project_id INTEGER PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
              remaining TEXT NOT NULL,
              cash_flow TEXT NOT NULL
            );
            CREATE TABLE location_cost (
              location_id INTEGER PRIMARY KEY REFERENCES location_node(id) ON DELETE CASCADE,
              amount TEXT NOT NULL
            );
            CREATE TABLE rate_library (
              id INTEGER PRIMARY KEY,
              kind TEXT NOT NULL CHECK (kind IN ('historical', 'market')),
              code TEXT NOT NULL,
              rate TEXT NOT NULL,
              UNIQUE (kind, code)
            );
            UPDATE app_meta SET value = '18' WHERE key = 'schema_version';
            ",
        )?;
    }
    if from < 19 {
        conn.execute_batch(
            "
            CREATE TABLE development_area (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              name TEXT NOT NULL,
              kind TEXT NOT NULL CHECK (kind IN ('saleable', 'common')),
              area TEXT NOT NULL
            );
            CREATE TABLE development_sale (
              project_id INTEGER PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
              rate_per_area TEXT NOT NULL
            );
            CREATE TABLE scenario (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              name TEXT NOT NULL,
              budget TEXT NOT NULL
            );
            CREATE TABLE final_account (
              project_id INTEGER PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
              certificate_id INTEGER NOT NULL REFERENCES ipc(id),
              amount TEXT NOT NULL
            );
            CREATE TABLE historical_cost (
              id INTEGER PRIMARY KEY,
              project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
              code TEXT NOT NULL,
              rate TEXT NOT NULL,
              amount TEXT NOT NULL
            );
            CREATE TABLE integration_setting (
              key TEXT PRIMARY KEY,
              enabled INTEGER NOT NULL
            );
            UPDATE app_meta SET value = '19' WHERE key = 'schema_version';
            ",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_firm_writes_schema_version_3() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("company.qsdb");
        let firm = create_firm(&path, "0.1.0").unwrap();
        assert!(path.exists());
        assert_eq!(firm.schema_version, 19);
        let stored: String = firm
            .conn
            .query_row(
                "SELECT value FROM app_meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, "19");
        let again = open_firm(&path, "0.1.0").unwrap();
        assert_eq!(again.schema_version, 19);
    }

    #[test]
    fn version_1_file_migrates_to_accounts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("company.qsdb");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE app_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO app_meta (key, value) VALUES ('schema_version', '1');
            ",
        )
        .unwrap();
        drop(conn);
        let firm = open_firm(&path, "0.1.0").unwrap();
        assert_eq!(firm.schema_version, 19);
        let tables: i64 = firm
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'user_account'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tables, 1);
    }

    #[test]
    fn create_refuses_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("company.qsdb");
        create_firm(&path, "0.1.0").unwrap();
        let error = create_firm(&path, "0.1.0").unwrap_err();
        assert!(error.contains("already exists"));
    }

    #[test]
    fn two_users_do_not_corrupt_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("company.qsdb");
        let first = create_firm(&path, "0.1.0").unwrap();
        let second = open_firm(&path, "0.1.0").unwrap();
        crate::project::save_project(&first.conn, "AAA", "First user").unwrap();
        crate::project::save_project(&second.conn, "BBB", "Second user").unwrap();
        drop(first);
        drop(second);
        let again = open_firm(&path, "0.1.0").unwrap();
        let count: i64 = again
            .conn
            .query_row("SELECT COUNT(*) FROM project", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
        assert_eq!(again.schema_version, SCHEMA_VERSION);
    }
}

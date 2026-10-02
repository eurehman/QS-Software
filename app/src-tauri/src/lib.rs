mod access;
mod audit;
mod backup;
mod codes;
mod ledger;
mod measure;
mod contract;
mod ipc;
mod cost;
mod variation;
mod estimate;
mod intermediate;
mod rate;
mod reconcile;
mod report;
mod project;
mod xlsx;
mod accounts;
mod advanced;
mod db;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager};

struct AppState {
    firm: Mutex<Option<db::FirmFile>>,
    session: Mutex<Option<accounts::Account>>,
    recent: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FirmStatus {
    path: String,
    schema_version: i64,
    app_version: String,
}

fn status_of(firm: &db::FirmFile) -> FirmStatus {
    FirmStatus {
        path: firm.path.display().to_string(),
        schema_version: firm.schema_version,
        app_version: firm.app_version.clone(),
    }
}

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn current_firm(state: tauri::State<AppState>) -> Result<Option<FirmStatus>, String> {
    let firm = state.firm.lock().map_err(|err| err.to_string())?;
    Ok(firm.as_ref().map(status_of))
}

#[tauri::command]
fn default_firm_path(app: AppHandle) -> Result<String, String> {
    let mut dir = app.path().document_dir().map_err(|err| err.to_string())?;
    dir.push("QS");
    dir.push("company.qsdb");
    Ok(dir.display().to_string())
}

#[tauri::command]
fn create_firm(state: tauri::State<AppState>, path: String) -> Result<FirmStatus, String> {
    let opened = db::create_firm(PathBuf::from(&path).as_path(), &app_version())?;
    remember(&state, opened)
}

#[tauri::command]
fn open_firm(state: tauri::State<AppState>, path: String) -> Result<FirmStatus, String> {
    let opened = db::open_firm(PathBuf::from(&path).as_path(), &app_version())?;
    remember(&state, opened)
}

fn remember(state: &tauri::State<AppState>, opened: db::FirmFile) -> Result<FirmStatus, String> {
    fs::write(&state.recent, opened.path.display().to_string()).map_err(|err| err.to_string())?;
    let status = status_of(&opened);
    let mut slot = state.firm.lock().map_err(|err| err.to_string())?;
    *slot = Some(opened);
    drop(slot);
    let mut session = state.session.lock().map_err(|err| err.to_string())?;
    *session = None;
    Ok(status)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionView {
    username: String,
    display_name: String,
    is_administrator: bool,
    role_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountStatus {
    needs_administrator: bool,
    session: Option<SessionView>,
    users: Vec<SessionView>,
    roles: Vec<access::Role>,
}

fn view_of(account: &accounts::Account) -> SessionView {
    SessionView {
        username: account.username.clone(),
        display_name: account.display_name.clone(),
        is_administrator: account.is_administrator,
        role_name: account.role_name.clone(),
    }
}

fn with_firm<T>(
    state: &tauri::State<AppState>,
    action: impl FnOnce(&db::FirmFile) -> Result<T, String>,
) -> Result<T, String> {
    let firm = state.firm.lock().map_err(|err| err.to_string())?;
    let open = firm.as_ref().ok_or("Open a firm file first.")?;
    action(open)
}

#[tauri::command]
fn account_status(state: tauri::State<AppState>) -> Result<AccountStatus, String> {
    let needs_administrator = with_firm(&state, |firm| {
        let count: i64 = firm
            .conn
            .query_row(
                "SELECT COUNT(*) FROM user_account WHERE is_administrator = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|err| err.to_string())?;
        Ok(count == 0)
    })?;
    let session = state.session.lock().map_err(|err| err.to_string())?;
    let session_view = session.as_ref().map(view_of);
    let is_administrator = session_view.as_ref().is_some_and(|account| account.is_administrator);
    drop(session);
    let (users, roles) = if is_administrator {
        let users = with_firm(&state, |firm| accounts::list_users(&firm.conn))?
            .iter()
            .map(view_of)
            .collect();
        let roles = with_firm(&state, |firm| access::list_roles(&firm.conn))?;
        (users, roles)
    } else {
        (Vec::new(), Vec::new())
    };
    Ok(AccountStatus {
        needs_administrator,
        session: session_view,
        users,
        roles,
    })
}

fn require_administrator(state: &tauri::State<AppState>) -> Result<(), String> {
    let session = state.session.lock().map_err(|err| err.to_string())?;
    if session.as_ref().is_some_and(|account| account.is_administrator) {
        Ok(())
    } else {
        Err("Sign in as the administrator.".into())
    }
}

#[tauri::command]
fn save_role(
    state: tauri::State<AppState>,
    name: String,
    permissions: Vec<String>,
    modules: Vec<String>,
    projects: Vec<String>,
) -> Result<(), String> {
    require_administrator(&state)?;
    with_firm(&state, |firm| {
        access::save_role(&firm.conn, &name, &permissions, &modules, &projects).map(|_| ())
    })
}

#[tauri::command]
fn assign_role(
    state: tauri::State<AppState>,
    username: String,
    role_name: String,
) -> Result<(), String> {
    require_administrator(&state)?;
    with_firm(&state, |firm| access::assign_role(&firm.conn, &username, &role_name))
}

#[tauri::command]
fn create_administrator(
    state: tauri::State<AppState>,
    username: String,
    password: String,
) -> Result<SessionView, String> {
    let account = with_firm(&state, |firm| {
        accounts::create_administrator(&firm.conn, &username, &password)
    })?;
    let view = view_of(&account);
    *state.session.lock().map_err(|err| err.to_string())? = Some(account);
    Ok(view)
}

#[tauri::command]
fn login(
    state: tauri::State<AppState>,
    username: String,
    password: String,
) -> Result<SessionView, String> {
    let account = with_firm(&state, |firm| {
        accounts::authenticate(&firm.conn, &username, &password)
    })?;
    let view = view_of(&account);
    *state.session.lock().map_err(|err| err.to_string())? = Some(account);
    Ok(view)
}

#[derive(Serialize, serde::Deserialize)]
struct SheetCell {
    row: i64,
    col: i64,
    raw: String,
}

fn session_account(state: &tauri::State<AppState>) -> Result<accounts::Account, String> {
    let session = state.session.lock().map_err(|err| err.to_string())?;
    session.clone().ok_or_else(|| "Sign in first.".into())
}

fn require_session(state: &tauri::State<AppState>) -> Result<(), String> {
    let session = state.session.lock().map_err(|err| err.to_string())?;
    if session.is_some() {
        Ok(())
    } else {
        Err("Sign in to use the sheet.".into())
    }
}

#[tauri::command]
fn load_sheet(state: tauri::State<AppState>) -> Result<Vec<SheetCell>, String> {
    require_session(&state)?;
    with_firm(&state, |firm| {
        let mut stmt = firm
            .conn
            .prepare("SELECT sheet_row, sheet_col, raw FROM sheet_cell")
            .map_err(|err| err.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(SheetCell {
                    row: row.get(0)?,
                    col: row.get(1)?,
                    raw: row.get(2)?,
                })
            })
            .map_err(|err| err.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|err| err.to_string())
    })
}

#[tauri::command]
fn save_sheet(state: tauri::State<AppState>, cells: Vec<SheetCell>) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        let stored: Vec<(i64, i64, String)> = cells
            .into_iter()
            .map(|cell| (cell.row, cell.col, cell.raw))
            .collect();
        audit::save_cells(&firm.conn, &account.username, &stored)
    })
}

#[tauri::command]
fn read_xlsx(state: tauri::State<AppState>, path: String) -> Result<Vec<Vec<String>>, String> {
    require_session(&state)?;
    xlsx::read_grid(std::path::Path::new(&path))
}

#[tauri::command]
fn write_xlsx(state: tauri::State<AppState>, path: String, rows: Vec<Vec<String>>) -> Result<(), String> {
    require_session(&state)?;
    xlsx::write_grid(std::path::Path::new(&path), &rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupStatus {
    folder: String,
    history: Vec<backup::BackupRecord>,
}

#[tauri::command]
fn backup_status(state: tauri::State<AppState>) -> Result<BackupStatus, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "backup", "security", None)?;
        Ok(BackupStatus {
            folder: backup::local_server(&firm.conn)?,
            history: backup::list_backups(&firm.conn)?,
        })
    })
}

#[tauri::command]
fn set_local_server(state: tauri::State<AppState>, folder: String) -> Result<(), String> {
    require_administrator(&state)?;
    with_firm(&state, |firm| backup::set_local_server(&firm.conn, &folder))
}

#[tauri::command]
fn create_backup(state: tauri::State<AppState>) -> Result<backup::BackupRecord, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "backup", "security", None)?;
        backup::create_backup(
            &firm.conn,
            &account.username,
            &app_version(),
            firm.schema_version,
            &firm.path.display().to_string(),
        )
    })
}

#[tauri::command]
fn restore_backup(state: tauri::State<AppState>, folder: String) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "restore", "security", None)?;
        backup::restore_backup(&firm.conn, &folder, &account.username)
    })
}

#[tauri::command]
fn list_audit(state: tauri::State<AppState>) -> Result<Vec<audit::AuditEvent>, String> {
    require_administrator(&state)?;
    with_firm(&state, |firm| audit::list_events(&firm.conn))
}

#[tauri::command]
fn load_company(state: tauri::State<AppState>) -> Result<project::Company, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "project", None)?;
        project::load_company(&firm.conn)
    })
}

#[tauri::command]
fn save_company(state: tauri::State<AppState>, name: String) -> Result<project::Company, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "edit", "project", None)?;
        project::save_company(&firm.conn, &name, &account.username)
    })
}

#[tauri::command]
fn list_projects(state: tauri::State<AppState>) -> Result<Vec<project::Project>, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "project", None)?;
        project::list_projects(&firm.conn)
    })
}

#[tauri::command]
fn save_project(state: tauri::State<AppState>, code: String, name: String) -> Result<project::Project, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "project", None)?;
        project::save_project_as(&firm.conn, &code, &name, &account.username)
    })
}

#[tauri::command]
fn list_locations(state: tauri::State<AppState>, project_code: String) -> Result<Vec<project::LocationNode>, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "project", Some(&project_code))?;
        project::list_locations(&firm.conn, &project_code)
    })
}

#[tauri::command]
fn add_location(
    state: tauri::State<AppState>,
    project_code: String,
    parent_id: Option<i64>,
    kind: String,
    label: String,
    name: String,
) -> Result<project::LocationNode, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "project", Some(&project_code))?;
        project::add_location(&firm.conn, &project_code, parent_id, &kind, &label, &name)
    })
}

#[tauri::command]
fn list_codes(state: tauri::State<AppState>, kind: String) -> Result<Vec<codes::CodeEntry>, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "project", None)?;
        codes::list_codes(&firm.conn, &kind)
    })
}

#[tauri::command]
fn save_code(
    state: tauri::State<AppState>,
    kind: String,
    code: String,
    name: String,
) -> Result<codes::CodeEntry, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "project", None)?;
        codes::save_code_as(&firm.conn, &kind, &code, &name, &account.username)
    })
}

#[tauri::command]
fn item_ledger(state: tauri::State<AppState>, item_id: i64) -> Result<ledger::QuantityLedger, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        let project_code: String = firm
            .conn
            .query_row(
                "SELECT project.code FROM work_item
                 JOIN project ON project.id = work_item.project_id
                 WHERE work_item.id = ?1",
                rusqlite::params![item_id],
                |row| row.get(0),
            )
            .map_err(|_| "That bill item was not found.".to_string())?;
        access::require_access(&firm.conn, &account, "view", "project", Some(&project_code))?;
        ledger::read(&firm.conn, item_id)
    })
}

#[tauri::command]
fn post_quantity(
    state: tauri::State<AppState>,
    item_id: i64,
    balance: String,
    quantity: String,
) -> Result<ledger::QuantityLedger, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        let project_code: String = firm
            .conn
            .query_row(
                "SELECT project.code FROM work_item
                 JOIN project ON project.id = work_item.project_id
                 WHERE work_item.id = ?1",
                rusqlite::params![item_id],
                |row| row.get(0),
            )
            .map_err(|_| "That bill item was not found.".to_string())?;
        access::require_access(&firm.conn, &account, "edit", "project", Some(&project_code))?;
        ledger::post_balance(&firm.conn, item_id, &balance, &quantity, &account.username)
    })
}

#[tauri::command]
fn list_items(state: tauri::State<AppState>, project_code: String) -> Result<Vec<codes::WorkItem>, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "project", Some(&project_code))?;
        codes::list_items(&firm.conn, &project_code)
    })
}

#[tauri::command]
fn save_item(
    state: tauri::State<AppState>,
    project_code: String,
    parent_id: Option<i64>,
    version_no: i64,
    name: String,
    quantity: String,
    rate: String,
    wbs_id: i64,
    cbs_id: i64,
    cost_id: i64,
    unit_id: i64,
    work_id: Option<i64>,
    discipline_id: Option<i64>,
    package_id: Option<i64>,
) -> Result<codes::WorkItem, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "project", Some(&project_code))?;
        codes::save_classified(
            &firm.conn,
            &project_code,
            parent_id,
            version_no,
            &name,
            &quantity,
            &rate,
            wbs_id,
            cbs_id,
            cost_id,
            unit_id,
            work_id,
            discipline_id,
            package_id,
            &account.username,
        )
    })
}

#[tauri::command]
fn revise_item_quantity(state: tauri::State<AppState>, item_id: i64, quantity: String) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        let project_code: String = firm
            .conn
            .query_row(
                "SELECT project.code FROM work_item
                 JOIN project ON project.id = work_item.project_id
                 WHERE work_item.id = ?1",
                rusqlite::params![item_id],
                |row| row.get(0),
            )
            .map_err(|_| "That bill item was not found.".to_string())?;
        access::require_access(&firm.conn, &account, "create", "project", Some(&project_code))?;
        codes::revise_quantity(&firm.conn, item_id, &quantity, &account.username)
    })
}

#[tauri::command]
fn list_measures(state: tauri::State<AppState>, item_id: i64) -> Result<Vec<measure::MeasureLine>, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "measure", None)?;
        measure::list_lines(&firm.conn, item_id)
    })
}

#[tauri::command]
fn add_measure(
    state: tauri::State<AppState>,
    item_id: i64,
    description: String,
    times: String,
    length: String,
    width: String,
    height: String,
) -> Result<measure::MeasureLine, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "measure", None)?;
        measure::add_line(&firm.conn, item_id, &description, &times, &length, &width, &height)
    })
}

#[tauri::command]
fn save_rate(
    state: tauri::State<AppState>,
    item_id: i64,
    material_qty: String,
    material_rate: String,
    labour_qty: String,
    labour_rate: String,
    plant_qty: String,
    plant_rate: String,
    wastage_percent: String,
    overhead_percent: String,
    profit_percent: String,
) -> Result<rate::RateBuildup, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "rates", None)?;
        rate::save_buildup(
            &firm.conn,
            item_id,
            &material_qty,
            &material_rate,
            &labour_qty,
            &labour_rate,
            &plant_qty,
            &plant_rate,
            &wastage_percent,
            &overhead_percent,
            &profit_percent,
        )
    })
}

#[tauri::command]
fn load_rate(state: tauri::State<AppState>, item_id: i64) -> Result<rate::RateBuildup, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "rates", None)?;
        rate::load(&firm.conn, item_id)
    })
}

#[tauri::command]
fn save_estimate(
    state: tauri::State<AppState>,
    project_code: String,
    kind: String,
    version_no: i64,
    area: String,
    rate: String,
) -> Result<estimate::Estimate, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "project", Some(&project_code))?;
        estimate::save_estimate(&firm.conn, &project_code, &kind, version_no, &area, &rate)
    })
}

#[tauri::command]
fn save_contractor(state: tauri::State<AppState>, name: String) -> Result<i64, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "contract", None)?;
        contract::save_contractor(&firm.conn, &name)
    })
}

#[tauri::command]
fn save_contract(
    state: tauri::State<AppState>,
    project_code: String,
    contractor_id: i64,
    code: String,
    name: String,
) -> Result<contract::ContractView, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "contract", Some(&project_code))?;
        contract::save_contract(&firm.conn, &project_code, contractor_id, &code, &name)
    })
}

#[tauri::command]
fn save_work_order(
    state: tauri::State<AppState>,
    contract_id: i64,
    code: String,
    name: String,
) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "contract", None)?;
        contract::save_work_order(&firm.conn, contract_id, &code, &name)
    })
}

#[tauri::command]
fn add_contract_item(
    state: tauri::State<AppState>,
    contract_id: i64,
    description: String,
    quantity: String,
    rate: String,
) -> Result<contract::ContractView, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "contract", None)?;
        contract::add_contract_item(&firm.conn, contract_id, &description, &quantity, &rate)
    })
}

#[tauri::command]
fn link_contract_item(
    state: tauri::State<AppState>,
    contract_id: i64,
    item_id: i64,
) -> Result<contract::ContractView, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "contract", None)?;
        contract::link_contract_item(&firm.conn, contract_id, item_id)
    })
}

#[tauri::command]
fn save_certificate(
    state: tauri::State<AppState>,
    contract_id: i64,
    certificate_no: i64,
    previous: String,
    work_to_date: String,
    retention_percent: String,
    advance_recovery: String,
    deductions: String,
) -> Result<ipc::Certificate, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "ipc", None)?;
        ipc::save_certificate(
            &firm.conn,
            contract_id,
            certificate_no,
            &previous,
            &work_to_date,
            &retention_percent,
            &advance_recovery,
            &deductions,
        )
    })
}

#[tauri::command]
fn save_variation(
    state: tauri::State<AppState>,
    contract_id: i64,
    kind: String,
    description: String,
    amount: String,
) -> Result<variation::VariationResult, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "variation", None)?;
        variation::save_variation(&firm.conn, contract_id, &kind, &description, &amount)
    })
}

#[tauri::command]
fn approve_variation(state: tauri::State<AppState>, variation_id: i64) -> Result<variation::VariationResult, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "approve", "variation", None)?;
        variation::approve_variation(&firm.conn, variation_id)
    })
}

#[tauri::command]
fn save_budget(state: tauri::State<AppState>, project_code: String, amount: String) -> Result<cost::CostPosition, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "cost", Some(&project_code))?;
        cost::save_budget(&firm.conn, &project_code, &amount)
    })
}

#[tauri::command]
fn project_report(state: tauri::State<AppState>, project_code: String) -> Result<Vec<report::ReportLine>, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "reports", Some(&project_code))?;
        report::screen_report(&firm.conn, &project_code)
    })
}

#[tauri::command]
fn export_report(state: tauri::State<AppState>, project_code: String, path: String) -> Result<Vec<report::ReportLine>, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "export", "reports", Some(&project_code))?;
        report::export_report(&firm.conn, &project_code, std::path::Path::new(&path))
    })
}

#[tauri::command]
fn export_core(state: tauri::State<AppState>, project_code: String, path: String) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "export", "reports", Some(&project_code))?;
        ledger::export_core(&firm.conn, &project_code, std::path::Path::new(&path))
    })
}

#[tauri::command]
fn reconcile_boq(
    state: tauri::State<AppState>,
    project_code: String,
    version_a: i64,
    version_b: i64,
) -> Result<reconcile::Reconciliation, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "boq", Some(&project_code))?;
        reconcile::reconcile(&firm.conn, &project_code, version_a, version_b)
    })
}

#[tauri::command]
fn approve_boq(state: tauri::State<AppState>, project_code: String, version_no: i64) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "approve", "boq", Some(&project_code))?;
        reconcile::approve_boq(&firm.conn, &project_code, version_no)
    })
}

#[tauri::command]
fn save_commitment(
    state: tauri::State<AppState>,
    project_code: String,
    description: String,
    order_amount: String,
    already_certified: String,
) -> Result<intermediate::Exposure, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "cost", Some(&project_code))?;
        intermediate::save_commitment(&firm.conn, &project_code, &description, &order_amount, &already_certified)
    })
}

#[tauri::command]
fn add_quote_line(
    state: tauri::State<AppState>,
    project_code: String,
    vendor: String,
    description: String,
    amount: String,
) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "contract", Some(&project_code))?;
        intermediate::add_quote_line(&firm.conn, &project_code, &vendor, &description, &amount)
    })
}

#[tauri::command]
fn comparative_statement(state: tauri::State<AppState>, project_code: String) -> Result<intermediate::Comparison, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "contract", Some(&project_code))?;
        intermediate::comparative_statement(&firm.conn, &project_code)
    })
}

#[tauri::command]
fn place_order(state: tauri::State<AppState>, project_code: String) -> Result<intermediate::Comparison, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "approve", "contract", Some(&project_code))?;
        intermediate::place_order(&firm.conn, &project_code)
    })
}

#[tauri::command]
fn save_material(
    state: tauri::State<AppState>,
    project_code: String,
    name: String,
    theoretical: String,
) -> Result<i64, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "cost", Some(&project_code))?;
        intermediate::save_material(&firm.conn, &project_code, &name, &theoretical)
    })
}

#[tauri::command]
fn add_material_move(
    state: tauri::State<AppState>,
    material_id: i64,
    kind: String,
    quantity: String,
) -> Result<intermediate::MaterialCheck, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "cost", None)?;
        intermediate::add_material_move(&firm.conn, material_id, &kind, &quantity)
    })
}

#[tauri::command]
fn save_forecast(
    state: tauri::State<AppState>,
    project_code: String,
    remaining: String,
    cash_flow: String,
) -> Result<intermediate::Forecast, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "cost", Some(&project_code))?;
        intermediate::save_forecast(&firm.conn, &project_code, &remaining, &cash_flow)
    })
}

#[tauri::command]
fn save_location_cost(state: tauri::State<AppState>, location_id: i64, amount: String) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "cost", None)?;
        intermediate::save_location_cost(&firm.conn, location_id, &amount)
    })
}

#[tauri::command]
fn location_rollup(state: tauri::State<AppState>, project_code: String) -> Result<intermediate::LocationRollup, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "cost", Some(&project_code))?;
        intermediate::location_rollup(&firm.conn, &project_code)
    })
}

#[tauri::command]
fn recommend_bill(state: tauri::State<AppState>, certificate_id: i64) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "recommend", "ipc", None)?;
        ipc::recommend_bill(&firm.conn, certificate_id)
    })
}

#[tauri::command]
fn approve_bill(state: tauri::State<AppState>, certificate_id: i64) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "approve", "ipc", None)?;
        ipc::approve_bill(&firm.conn, certificate_id)
    })
}

#[tauri::command]
fn reject_bill(state: tauri::State<AppState>, certificate_id: i64) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "reject", "ipc", None)?;
        ipc::reject_bill(&firm.conn, certificate_id)
    })
}

#[tauri::command]
fn certify_bill(state: tauri::State<AppState>, certificate_id: i64) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "approve", "ipc", None)?;
        ipc::certify_bill(&firm.conn, certificate_id)
    })
}

#[tauri::command]
fn save_library_rate(state: tauri::State<AppState>, kind: String, code: String, rate: String) -> Result<i64, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "rates", None)?;
        intermediate::save_library_rate(&firm.conn, &kind, &code, &rate)
    })
}

#[tauri::command]
fn apply_library_rate(state: tauri::State<AppState>, item_id: i64, library_id: i64) -> Result<String, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "edit", "rates", None)?;
        intermediate::apply_library_rate(&firm.conn, item_id, library_id)
    })
}

#[tauri::command]
fn project_kpi(state: tauri::State<AppState>, project_code: String) -> Result<intermediate::Kpi, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "cost", Some(&project_code))?;
        intermediate::project_kpi(&firm.conn, &project_code)
    })
}

#[tauri::command]
fn save_area(
    state: tauri::State<AppState>,
    project_code: String,
    name: String,
    kind: String,
    area: String,
) -> Result<advanced::AreaBuildup, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "project", Some(&project_code))?;
        advanced::save_area(&firm.conn, &project_code, &name, &kind, &area)
    })
}

#[tauri::command]
fn save_sale_rate(state: tauri::State<AppState>, project_code: String, rate: String) -> Result<advanced::AreaBuildup, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "project", Some(&project_code))?;
        advanced::save_sale_rate(&firm.conn, &project_code, &rate)
    })
}

#[tauri::command]
fn save_scenario(
    state: tauri::State<AppState>,
    project_code: String,
    name: String,
    budget: String,
) -> Result<advanced::ScenarioView, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "cost", Some(&project_code))?;
        advanced::save_scenario(&firm.conn, &project_code, &name, &budget)
    })
}

#[tauri::command]
fn close_account(state: tauri::State<AppState>, project_code: String) -> Result<advanced::FinalAccount, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "approve", "cost", Some(&project_code))?;
        advanced::close_account(&firm.conn, &project_code)
    })
}

#[tauri::command]
fn project_totals(state: tauri::State<AppState>) -> Result<Vec<advanced::ProjectTotal>, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "view", "cost", None)?;
        advanced::project_totals(&firm.conn)
    })
}

#[tauri::command]
fn set_integration(state: tauri::State<AppState>, key: String, enabled: bool) -> Result<bool, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "configure", "project", None)?;
        advanced::set_integration(&firm.conn, &key, enabled)
    })
}

#[tauri::command]
fn export_local_workbook(state: tauri::State<AppState>, project_code: String, path: String) -> Result<(), String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "export", "reports", Some(&project_code))?;
        advanced::export_local_workbook(&firm.conn, &project_code, std::path::Path::new(&path))
    })
}

#[tauri::command]
fn logout(state: tauri::State<AppState>) -> Result<(), String> {
    *state.session.lock().map_err(|err| err.to_string())? = None;
    Ok(())
}

#[tauri::command]
fn create_user(
    state: tauri::State<AppState>,
    username: String,
    display_name: String,
    password: String,
) -> Result<(), String> {
    let session = state.session.lock().map_err(|err| err.to_string())?;
    if !session.as_ref().is_some_and(|account| account.is_administrator) {
        return Err("Sign in as the administrator to add a user.".into());
    }
    drop(session);
    with_firm(&state, |firm| {
        accounts::create_user(&firm.conn, &username, &display_name, &password).map(|_| ())
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let mut recent = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
            fs::create_dir_all(&recent).ok();
            recent.push("recent.txt");
            let state = AppState {
                firm: Mutex::new(None),
                session: Mutex::new(None),
                recent: recent.clone(),
            };
            app.manage(state);
            if let Ok(saved) = fs::read_to_string(&recent) {
                let path = PathBuf::from(saved.trim());
                if path.exists() {
                    if let Ok(opened) = db::open_firm(&path, &app_version()) {
                        let managed = app.state::<AppState>();
                        if let Ok(mut slot) = managed.firm.lock() {
                            *slot = Some(opened);
                        };
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            current_firm,
            default_firm_path,
            create_firm,
            open_firm,
            account_status,
            create_administrator,
            login,
            logout,
            create_user,
            save_role,
            assign_role,
            load_sheet,
            save_sheet,
            read_xlsx,
            write_xlsx,
            backup_status,
            set_local_server,
            create_backup,
            restore_backup,
            list_audit,
            load_company,
            save_company,
            list_projects,
            save_project,
            list_locations,
            add_location,
            list_codes,
            save_code,
            list_items,
            item_ledger,
            post_quantity,
            save_item,
            revise_item_quantity,
            list_measures,
            add_measure,
            save_rate,
            load_rate,
            save_estimate,
            save_contractor,
            save_contract,
            save_work_order,
            add_contract_item,
            link_contract_item,
            save_certificate,
            save_variation,
            approve_variation,
            save_budget,
            project_report,
            export_report,
            export_core,
            reconcile_boq,
            approve_boq,
            save_commitment,
            add_quote_line,
            comparative_statement,
            place_order,
            save_material,
            add_material_move,
            save_forecast,
            save_location_cost,
            location_rollup,
            recommend_bill,
            approve_bill,
            reject_bill,
            certify_bill,
            save_library_rate,
            apply_library_rate,
            project_kpi,
            save_area,
            save_sale_rate,
            save_scenario,
            close_account,
            project_totals,
            set_integration,
            export_local_workbook
        ])
        .run(tauri::generate_context!())
        .expect("error while running QS");
}

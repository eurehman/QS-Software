mod access;
mod audit;
mod backup;
mod codes;
mod measure;
mod contract;
mod ipc;
mod estimate;
mod rate;
mod project;
mod xlsx;
mod accounts;
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
        project::save_project(&firm.conn, &code, &name)
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
    label: String,
    name: String,
) -> Result<project::LocationNode, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "project", Some(&project_code))?;
        project::add_location(&firm.conn, &project_code, parent_id, &label, &name)
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
        codes::save_code(&firm.conn, &kind, &code, &name)
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
) -> Result<codes::WorkItem, String> {
    let account = session_account(&state)?;
    with_firm(&state, |firm| {
        access::require_access(&firm.conn, &account, "create", "project", Some(&project_code))?;
        codes::save_item(
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
        )
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
            list_projects,
            save_project,
            list_locations,
            add_location,
            list_codes,
            save_code,
            list_items,
            save_item,
            list_measures,
            add_measure,
            save_rate,
            load_rate,
            save_estimate,
            save_contractor,
            save_contract,
            save_work_order,
            add_contract_item,
            save_certificate
        ])
        .run(tauri::generate_context!())
        .expect("error while running QS");
}

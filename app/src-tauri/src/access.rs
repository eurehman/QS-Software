use rusqlite::{params, Connection};
use serde::Serialize;

use crate::accounts::Account;

pub const PERMISSIONS: &[&str] = &[
    "view", "create", "edit", "delete", "import", "export", "approve", "reject", "recommend",
    "print", "backup", "restore", "configure",
];

pub const MODULES: &[&str] = &[
    "project", "boq", "measure", "rates", "contract", "ipc", "variation", "cost", "reports",
    "security",
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub permissions: Vec<String>,
    pub modules: Vec<String>,
    pub projects: Vec<String>,
}

pub fn save_role(
    conn: &Connection,
    name: &str,
    permissions: &[String],
    modules: &[String],
    projects: &[String],
) -> Result<Role, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Enter a role name.".into());
    }
    for permission in permissions {
        if !PERMISSIONS.contains(&permission.as_str()) {
            return Err(format!("Unknown permission: {permission}"));
        }
    }
    for module in modules {
        if !MODULES.contains(&module.as_str()) {
            return Err(format!("Unknown module: {module}"));
        }
    }
    let tx = conn.unchecked_transaction().map_err(|err| err.to_string())?;
    tx.execute(
        "INSERT INTO role (name) VALUES (?1)
         ON CONFLICT(name) DO UPDATE SET name = excluded.name",
        params![name],
    )
    .map_err(|err| err.to_string())?;
    let id: i64 = tx
        .query_row("SELECT id FROM role WHERE name = ?1", params![name], |row| row.get(0))
        .map_err(|err| err.to_string())?;
    tx.execute("DELETE FROM role_permission WHERE role_id = ?1", params![id])
        .map_err(|err| err.to_string())?;
    tx.execute("DELETE FROM role_module WHERE role_id = ?1", params![id])
        .map_err(|err| err.to_string())?;
    tx.execute("DELETE FROM role_project WHERE role_id = ?1", params![id])
        .map_err(|err| err.to_string())?;
    for permission in permissions {
        tx.execute(
            "INSERT INTO role_permission (role_id, permission) VALUES (?1, ?2)",
            params![id, permission],
        )
        .map_err(|err| err.to_string())?;
    }
    for module in modules {
        tx.execute(
            "INSERT INTO role_module (role_id, module) VALUES (?1, ?2)",
            params![id, module],
        )
        .map_err(|err| err.to_string())?;
    }
    for project in projects {
        let project = project.trim();
        if project.is_empty() {
            continue;
        }
        tx.execute(
            "INSERT INTO role_project (role_id, project_code) VALUES (?1, ?2)",
            params![id, project],
        )
        .map_err(|err| err.to_string())?;
    }
    tx.commit().map_err(|err| err.to_string())?;
    Ok(Role {
        id,
        name: name.to_string(),
        permissions: permissions.to_vec(),
        modules: modules.to_vec(),
        projects: projects.iter().map(|item| item.trim().to_string()).filter(|item| !item.is_empty()).collect(),
    })
}

pub fn assign_role(conn: &Connection, username: &str, role_name: &str) -> Result<(), String> {
    let role_id: i64 = conn
        .query_row(
            "SELECT id FROM role WHERE name = ?1",
            params![role_name.trim()],
            |row| row.get(0),
        )
        .map_err(|_| format!("Role {role_name} was not found."))?;
    let changed = conn
        .execute(
            "UPDATE user_account SET role_id = ?1 WHERE username = ?2 AND is_administrator = 0",
            params![role_id, username.trim()],
        )
        .map_err(|err| err.to_string())?;
    if changed == 0 {
        return Err("That user cannot be assigned a role.".into());
    }
    Ok(())
}

pub fn list_roles(conn: &Connection) -> Result<Vec<Role>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM role ORDER BY name")
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
        .map_err(|err| err.to_string())?;
    let mut roles = Vec::new();
    for row in rows {
        let (id, name) = row.map_err(|err| err.to_string())?;
        roles.push(Role {
            id,
            name,
            permissions: column_list(conn, "role_permission", "permission", id)?,
            modules: column_list(conn, "role_module", "module", id)?,
            projects: column_list(conn, "role_project", "project_code", id)?,
        });
    }
    Ok(roles)
}

/// Administrator may do anything. Any other account must hold the permission, the module, and the project when one is named.
pub fn require_access(
    conn: &Connection,
    account: &Account,
    permission: &str,
    module: &str,
    project: Option<&str>,
) -> Result<(), String> {
    if account.is_administrator {
        return Ok(());
    }
    let role_id: Option<i64> = conn
        .query_row(
            "SELECT role_id FROM user_account WHERE id = ?1",
            params![account.id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    let Some(role_id) = role_id else {
        return Err("This account has no role, so it cannot act.".into());
    };
    let allowed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM role_permission WHERE role_id = ?1 AND permission = ?2",
            params![role_id, permission],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if allowed == 0 {
        return Err(format!("This account cannot {permission}."));
    }
    let module_ok: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM role_module WHERE role_id = ?1 AND module = ?2",
            params![role_id, module],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if module_ok == 0 {
        return Err(format!("This account cannot open {module}."));
    }
    if let Some(project) = project {
        let project_ok: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM role_project WHERE role_id = ?1 AND project_code = ?2",
                params![role_id, project],
                |row| row.get(0),
            )
            .map_err(|err| err.to_string())?;
        if project_ok == 0 {
            return Err(format!("This account cannot open project {project}."));
        }
    }
    Ok(())
}

fn column_list(conn: &Connection, table: &str, column: &str, role_id: i64) -> Result<Vec<String>, String> {
    let sql = format!("SELECT {column} FROM {table} WHERE role_id = ?1 ORDER BY {column}");
    let mut stmt = conn.prepare(&sql).map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![role_id], |row| row.get::<_, String>(0))
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{authenticate, create_administrator, create_user};
    use crate::db::create_firm;

    #[test]
    fn user_without_the_right_cannot_act() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("company.qsdb");
        let firm = create_firm(&path, "0.1.0").unwrap();
        let admin = create_administrator(&firm.conn, "admin", "password1").unwrap();
        let user = create_user(&firm.conn, "billing", "Billing QS", "password1").unwrap();
        save_role(
            &firm.conn,
            "Billing",
            &["view".into()],
            &["ipc".into()],
            &["Tower A".into()],
        )
        .unwrap();
        assign_role(&firm.conn, "billing", "Billing").unwrap();
        let signed_in = authenticate(&firm.conn, "billing", "password1").unwrap();
        let denied = require_access(&firm.conn, &signed_in, "create", "ipc", Some("Tower A")).unwrap_err();
        assert!(denied.contains("cannot create"));
        require_access(&firm.conn, &signed_in, "view", "ipc", Some("Tower A")).unwrap();
        let other_project = require_access(&firm.conn, &signed_in, "view", "ipc", Some("Tower B")).unwrap_err();
        assert!(other_project.contains("Tower B"));
        require_access(&firm.conn, &admin, "delete", "boq", Some("Tower B")).unwrap();
        let _ = user;
    }
}

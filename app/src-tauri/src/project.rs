use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationNode {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub kind: String,
    pub label: String,
    pub name: String,
}

const LOCATION_KINDS: &[&str] = &["development", "building", "tower", "floor", "unit", "zone", "custom"];

pub fn save_project(conn: &Connection, code: &str, name: &str) -> Result<Project, String> {
    save_project_as(conn, code, name, "")
}

pub fn save_project_as(conn: &Connection, code: &str, name: &str, username: &str) -> Result<Project, String> {
    let code = code.trim();
    let name = name.trim();
    if code.is_empty() || name.is_empty() {
        return Err("Enter a project code and name.".into());
    }
    let previous = match conn.query_row(
        "SELECT name FROM project WHERE code = ?1",
        params![code],
        |row| row.get::<_, String>(0),
    ) {
        Ok(stored) => Some(stored),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(err) => return Err(err.to_string()),
    };
    conn.execute(
        "INSERT INTO project (code, name) VALUES (?1, ?2)
         ON CONFLICT(code) DO UPDATE SET name = excluded.name",
        params![code, name],
    )
    .map_err(|err| err.to_string())?;
    let old = previous.clone().unwrap_or_default();
    let action = if previous.is_none() { "create" } else { "edit" };
    crate::audit::record(conn, username, action, &format!("project:{code}"), &old, name)?;
    load_project(conn, code)
}

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>, String> {
    let mut stmt = conn
        .prepare("SELECT id, code, name FROM project ORDER BY code")
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|err| err.to_string())
}

pub fn add_location(
    conn: &Connection,
    project_code: &str,
    parent_id: Option<i64>,
    kind: &str,
    label: &str,
    name: &str,
) -> Result<LocationNode, String> {
    let label = label.trim();
    let name = name.trim();
    if label.is_empty() || name.is_empty() {
        return Err("Enter a location label and name.".into());
    }
    let kind = location_kind(kind, label)?;
    let project = load_project(conn, project_code.trim())?;
    if let Some(parent_id) = parent_id {
        let parent_project: i64 = conn
            .query_row(
                "SELECT project_id FROM location_node WHERE id = ?1",
                params![parent_id],
                |row| row.get(0),
            )
            .map_err(|_| "That parent location was not found.".to_string())?;
        if parent_project != project.id {
            return Err("The parent location belongs to another project.".into());
        }
    }
    conn.execute(
        "INSERT INTO location_node (project_id, parent_id, kind, label, name) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![project.id, parent_id, kind, label, name],
    )
    .map_err(|err| err.to_string())?;
    Ok(LocationNode {
        id: conn.last_insert_rowid(),
        parent_id,
        kind,
        label: label.to_string(),
        name: name.to_string(),
    })
}

fn location_kind(kind: &str, label: &str) -> Result<String, String> {
    let chosen = kind.trim().to_lowercase();
    let kind = if chosen.is_empty() {
        let inferred = label.trim().to_lowercase();
        if LOCATION_KINDS.contains(&inferred.as_str()) {
            inferred
        } else {
            "custom".into()
        }
    } else {
        chosen
    };
    if LOCATION_KINDS.contains(&kind.as_str()) {
        Ok(kind)
    } else {
        Err("Choose development, building, tower, floor, unit, zone, or custom.".into())
    }
}

pub fn list_locations(conn: &Connection, project_code: &str) -> Result<Vec<LocationNode>, String> {
    let project = load_project(conn, project_code.trim())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, parent_id, kind, label, name FROM location_node
             WHERE project_id = ?1 ORDER BY id",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![project.id], |row| {
            Ok(LocationNode {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                kind: row.get(2)?,
                label: row.get(3)?,
                name: row.get(4)?,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|err| err.to_string())
}

fn load_project(conn: &Connection, code: &str) -> Result<Project, String> {
    conn.query_row(
        "SELECT id, code, name FROM project WHERE code = ?1",
        params![code],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
            })
        },
    )
    .map_err(|_| format!("Project {code} was not found."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_firm;

    #[test]
    fn two_different_tree_shapes_save() {
        let dir = tempfile::tempdir().unwrap();
        let firm = create_firm(&dir.path().join("company.qsdb"), "0.1.0").unwrap();
        save_project(&firm.conn, "TWR", "Tower site").unwrap();
        let north = add_location(&firm.conn, "TWR", None, "", "Development", "North").unwrap();
        let tower = add_location(&firm.conn, "TWR", Some(north.id), "", "Tower", "A").unwrap();
        add_location(&firm.conn, "TWR", Some(tower.id), "", "Floor", "1").unwrap();

        save_project(&firm.conn, "LIN", "Linear works").unwrap();
        let phase = add_location(&firm.conn, "LIN", None, "", "Phase", "1").unwrap();
        add_location(&firm.conn, "LIN", Some(phase.id), "", "Zone", "East").unwrap();

        let tower_tree = list_locations(&firm.conn, "TWR").unwrap();
        assert_eq!(
            tower_tree.iter().map(|node| node.label.as_str()).collect::<Vec<_>>(),
            vec!["Development", "Tower", "Floor"]
        );
        assert_eq!(tower_tree[2].parent_id, Some(tower_tree[1].id));

        let linear = list_locations(&firm.conn, "LIN").unwrap();
        assert_eq!(
            linear.iter().map(|node| node.label.as_str()).collect::<Vec<_>>(),
            vec!["Phase", "Zone"]
        );
        assert_eq!(linear[1].parent_id, Some(linear[0].id));
        assert_ne!(tower_tree.len(), linear.len());
    }

    #[test]
    fn tower_floor_and_unit_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("company.qsdb");
        let firm = create_firm(&path, "0.1.0").unwrap();
        save_project(&firm.conn, "HOME", "Harbour view").unwrap();
        let tower = add_location(&firm.conn, "HOME", None, "tower", "Block", "T1").unwrap();
        let floor = add_location(&firm.conn, "HOME", Some(tower.id), "floor", "Level", "03").unwrap();
        add_location(&firm.conn, "HOME", Some(floor.id), "unit", "Apartment", "301").unwrap();
        drop(firm);
        let again = crate::db::open_firm(&path, "0.1.0").unwrap();
        let nodes = list_locations(&again.conn, "HOME").unwrap();
        assert_eq!(
            nodes.iter().map(|node| node.kind.as_str()).collect::<Vec<_>>(),
            vec!["tower", "floor", "unit"]
        );
        assert_eq!(nodes[1].parent_id, Some(nodes[0].id));
        assert_eq!(nodes[2].parent_id, Some(nodes[1].id));
        assert_eq!(nodes[0].label, "Block");
        assert_eq!(nodes[2].name, "301");
        let error = add_location(&again.conn, "HOME", None, "wing", "Wing", "West").unwrap_err();
        assert!(error.contains("custom"));
    }
}

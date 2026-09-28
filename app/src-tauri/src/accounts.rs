use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand_core::OsRng;
use rusqlite::{params, Connection};

pub const MAX_ADDITIONAL_USERS: i64 = 4;

#[derive(Clone, Debug)]
pub struct Account {
    #[allow(dead_code)]
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub is_administrator: bool,
    pub role_name: Option<String>,
}

pub fn create_administrator(
    conn: &Connection,
    username: &str,
    password: &str,
) -> Result<Account, String> {
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_account WHERE is_administrator = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if existing > 0 {
        return Err("The firm already has an administrator.".into());
    }
    insert_account(conn, username, username, password, true)
}

pub fn create_user(
    conn: &Connection,
    username: &str,
    display_name: &str,
    password: &str,
) -> Result<Account, String> {
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_account WHERE is_administrator = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if existing >= MAX_ADDITIONAL_USERS {
        return Err(
            "The firm already has the administrator and four users. Another account is not allowed."
                .into(),
        );
    }
    insert_account(conn, username, display_name, password, false)
}

pub fn authenticate(conn: &Connection, username: &str, password: &str) -> Result<Account, String> {
    let row = conn.query_row(
        "SELECT u.id, u.username, u.display_name, u.is_administrator, u.password_hash, r.name
         FROM user_account u
         LEFT JOIN role r ON r.id = u.role_id
         WHERE u.username = ?1",
        params![username.trim()],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        },
    );
    let (id, stored_name, display_name, is_administrator, hash, role_name) = match row {
        Ok(row) => row,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err("The username or password is not correct.".into());
        }
        Err(err) => return Err(err.to_string()),
    };
    if !verify_password(password, &hash)? {
        return Err("The username or password is not correct.".into());
    }
    Ok(Account {
        id,
        username: stored_name,
        display_name,
        is_administrator: is_administrator == 1,
        role_name,
    })
}

pub fn list_users(conn: &Connection) -> Result<Vec<Account>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.username, u.display_name, u.is_administrator, r.name
             FROM user_account u
             LEFT JOIN role r ON r.id = u.role_id
             ORDER BY u.is_administrator DESC, u.username",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Account {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                is_administrator: row.get::<_, i64>(3)? == 1,
                role_name: row.get(4)?,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|err| err.to_string())
}

fn insert_account(
    conn: &Connection,
    username: &str,
    display_name: &str,
    password: &str,
    is_administrator: bool,
) -> Result<Account, String> {
    let username = clean_username(username)?;
    let display_name = display_name.trim();
    if display_name.is_empty() {
        return Err("Enter a name.".into());
    }
    check_password(password)?;
    let hash = hash_password(password)?;
    conn.execute(
        "INSERT INTO user_account (username, display_name, password_hash, is_administrator)
         VALUES (?1, ?2, ?3, ?4)",
        params![username, display_name, hash, if is_administrator { 1 } else { 0 }],
    )
    .map_err(|err| {
        if err.to_string().contains("UNIQUE") {
            if is_administrator {
                "The firm already has an administrator.".into()
            } else {
                "That username is already in use.".into()
            }
        } else {
            err.to_string()
        }
    })?;
    let id = conn.last_insert_rowid();
    Ok(Account {
        id,
        username,
        display_name: display_name.to_string(),
        is_administrator,
        role_name: None,
    })
}

fn clean_username(username: &str) -> Result<String, String> {
    let username = username.trim();
    if username.is_empty() || username.len() > 64 || username.chars().any(char::is_control) {
        return Err("Enter a username of up to 64 characters.".into());
    }
    Ok(username.to_string())
}

fn check_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Use a password of at least 8 characters.".into());
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| err.to_string())
}

fn verify_password(password: &str, stored: &str) -> Result<bool, String> {
    let parsed = PasswordHash::new(stored).map_err(|err| err.to_string())?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_firm, open_firm};

    #[test]
    fn sixth_account_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("company.qsdb");
        let firm = create_firm(&path, "0.1.0").unwrap();
        create_administrator(&firm.conn, "admin", "password1").unwrap();
        for index in 1..=4 {
            create_user(
                &firm.conn,
                &format!("user{index}"),
                &format!("User {index}"),
                "password1",
            )
            .unwrap();
        }
        let error = create_user(&firm.conn, "user5", "User 5", "password1").unwrap_err();
        assert!(error.contains("not allowed"));
        let again = open_firm(&path, "0.1.0").unwrap();
        let count: i64 = again
            .conn
            .query_row("SELECT COUNT(*) FROM user_account", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn second_administrator_is_rejected_and_password_must_match() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("company.qsdb");
        let firm = create_firm(&path, "0.1.0").unwrap();
        create_administrator(&firm.conn, "admin", "password1").unwrap();
        let error = create_administrator(&firm.conn, "other", "password1").unwrap_err();
        assert!(error.contains("administrator"));
        assert!(authenticate(&firm.conn, "admin", "wrong-password").is_err());
        let signed_in = authenticate(&firm.conn, "admin", "password1").unwrap();
        assert!(signed_in.is_administrator);
    }
}

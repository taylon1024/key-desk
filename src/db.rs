use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::{NewVariable, Variable};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS variables (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    scope TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_secret INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(scope, key)
);
";

#[derive(Debug)]
pub enum DbError {
    NotFound,
    Conflict,
    Other(rusqlite::Error),
}

pub fn db_path() -> PathBuf {
    std::env::var("KEY_DESK_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/variables.db"))
}

pub fn open(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let conn = Connection::open(path).map_err(|err| err.to_string())?;
    conn.execute_batch(SCHEMA).map_err(|err| err.to_string())?;
    Ok(conn)
}

pub fn list(conn: &Connection, scope: Option<&str>) -> Result<Vec<Variable>, DbError> {
    let mut statement = conn
        .prepare(
            "SELECT id, key, value, scope, description, is_secret, created_at, updated_at
             FROM variables
             WHERE (?1 IS NULL OR scope = ?1)
             ORDER BY scope, key",
        )
        .map_err(DbError::Other)?;
    let rows = statement
        .query_map(params![scope], map_variable)
        .map_err(DbError::Other)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::Other)
}

pub fn insert(conn: &Connection, input: &NewVariable) -> Result<Variable, DbError> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO variables (key, value, scope, description, is_secret, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            input.key,
            input.value,
            input.scope,
            input.description,
            input.is_secret as i64,
            now,
        ],
    )
    .map_err(map_write_error)?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, input: &NewVariable) -> Result<Variable, DbError> {
    let now = Utc::now().to_rfc3339();
    let changed = conn
        .execute(
            "UPDATE variables
             SET key = ?1, value = ?2, scope = ?3, description = ?4, is_secret = ?5, updated_at = ?6
             WHERE id = ?7",
            params![
                input.key,
                input.value,
                input.scope,
                input.description,
                input.is_secret as i64,
                now,
                id,
            ],
        )
        .map_err(map_write_error)?;
    if changed == 0 {
        return Err(DbError::NotFound);
    }
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), DbError> {
    let changed = conn
        .execute("DELETE FROM variables WHERE id = ?1", [id])
        .map_err(DbError::Other)?;
    if changed == 0 {
        Err(DbError::NotFound)
    } else {
        Ok(())
    }
}

fn get(conn: &Connection, id: i64) -> Result<Variable, DbError> {
    conn.query_row(
        "SELECT id, key, value, scope, description, is_secret, created_at, updated_at
         FROM variables WHERE id = ?1",
        [id],
        map_variable,
    )
    .optional()
    .map_err(DbError::Other)?
    .ok_or(DbError::NotFound)
}

fn map_variable(row: &rusqlite::Row<'_>) -> rusqlite::Result<Variable> {
    Ok(Variable {
        id: row.get(0)?,
        key: row.get(1)?,
        value: row.get(2)?,
        scope: row.get(3)?,
        description: row.get(4)?,
        is_secret: row.get::<_, i64>(5)? != 0,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_write_error(err: rusqlite::Error) -> DbError {
    match &err {
        rusqlite::Error::SqliteFailure(sqlite_err, _)
            if sqlite_err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            DbError::Conflict
        }
        _ => DbError::Other(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (Connection, PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "key-desk-{}-{}.db",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let conn = open(&path).expect("open test db");
        (conn, path)
    }

    #[test]
    fn inserts_lists_and_rejects_duplicates() {
        let (conn, path) = temp_db();
        let input = NewVariable {
            key: "API_TOKEN".to_string(),
            value: "secret".to_string(),
            scope: "dev".to_string(),
            description: String::new(),
            is_secret: true,
        };
        let created = insert(&conn, &input).unwrap();
        assert!(created.is_secret);
        assert_eq!(list(&conn, Some("dev")).unwrap().len(), 1);
        assert!(matches!(insert(&conn, &input), Err(DbError::Conflict)));
        delete(&conn, created.id).unwrap();
        assert!(list(&conn, None).unwrap().is_empty());
        let _ = std::fs::remove_file(path);
    }
}

use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};

use crate::crypto;
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
    Crypto(String),
    Other(rusqlite::Error),
}

pub fn db_path() -> PathBuf {
    if let Ok(path) = std::env::var("KEY_DESK_DB") {
        return PathBuf::from(path);
    }
    let path = support_dir().join("variables.db");
    copy_legacy_store(&path);
    path
}

fn support_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        home_dir().join("Library/Application Support/key-desk")
    }
    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir().join("AppData").join("Roaming"))
            .join("key-desk");
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        home_dir().join(".local/share/key-desk")
    }
}

fn home_dir() -> PathBuf {
    #[cfg(windows)]
    let var = "USERPROFILE";
    #[cfg(not(windows))]
    let var = "HOME";
    std::env::var(var)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// 开发期数据库在仓库 `data/`。正式目录还没有库时复制一份，原文件保留。
fn copy_legacy_store(dest: &Path) {
    if dest.exists() {
        return;
    }
    let legacy = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/variables.db");
    if !legacy.exists() {
        return;
    }
    if let Some(parent) = dest.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        log::warn!("could not create {}", parent.display());
        return;
    }
    if std::fs::copy(&legacy, dest).is_err() {
        log::warn!(
            "could not copy vault from {} to {}",
            legacy.display(),
            dest.display()
        );
        return;
    }
    copy_if_missing(
        PathBuf::from(format!("{}.key", legacy.display())),
        PathBuf::from(format!("{}.key", dest.display())),
    );
    if let (Some(from_dir), Some(to_dir)) = (legacy.parent(), dest.parent()) {
        copy_if_missing(from_dir.join("theme_id"), to_dir.join("theme_id"));
    }
    log::info!(
        "copied vault from {} to {}",
        legacy.display(),
        dest.display()
    );
}

fn copy_if_missing(from: PathBuf, to: PathBuf) {
    if !from.exists() || to.exists() {
        return;
    }
    if std::fs::copy(&from, &to).is_err() {
        log::warn!("could not copy {} to {}", from.display(), to.display());
        return;
    }
    #[cfg(unix)]
    if to.extension().and_then(|ext| ext.to_str()) == Some("key") {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&to, std::fs::Permissions::from_mode(0o600));
    }
}

pub fn open(path: &Path) -> Result<Connection, String> {
    log::info!("opening database {}", path.display());
    match open_database(path) {
        Ok(connection) => {
            log::debug!("database ready");
            Ok(connection)
        }
        Err(err) => {
            log::error!("database open failed: {err}");
            Err(err)
        }
    }
}

fn open_database(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    crypto::bind_store(path);
    let conn = Connection::open(path).map_err(|err| err.to_string())?;
    conn.execute_batch(SCHEMA).map_err(|err| err.to_string())?;
    restrict_db_file(path);
    migrate_plaintext(&conn)?;
    Ok(conn)
}

fn restrict_db_file(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
}

/// 把还没加密的旧 value 改写成密文。已经是 `kd1:` 的行不动。
fn migrate_plaintext(conn: &Connection) -> Result<(), String> {
    let mut statement = conn
        .prepare("SELECT id, value FROM variables")
        .map_err(|err| err.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    for (id, value) in rows {
        if crypto::is_sealed(&value) {
            continue;
        }
        let sealed = crypto::seal(&value)?;
        conn.execute(
            "UPDATE variables SET value = ?1 WHERE id = ?2",
            params![sealed, id],
        )
        .map_err(|err| err.to_string())?;
    }
    Ok(())
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
        .map_err(map_read_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_read_error)
}

pub fn insert(conn: &Connection, input: &NewVariable) -> Result<Variable, DbError> {
    let now = Utc::now().to_rfc3339();
    let value = crypto::seal(&input.value).map_err(DbError::Crypto)?;
    conn.execute(
        "INSERT INTO variables (key, value, scope, description, is_secret, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            input.key,
            value,
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
    let value = crypto::seal(&input.value).map_err(DbError::Crypto)?;
    let changed = conn
        .execute(
            "UPDATE variables
             SET key = ?1, value = ?2, scope = ?3, description = ?4, is_secret = ?5, updated_at = ?6
             WHERE id = ?7",
            params![
                input.key,
                value,
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

pub fn find_in_scope(
    conn: &Connection,
    scope: &str,
    key: &str,
) -> Result<Option<Variable>, DbError> {
    conn.query_row(
        "SELECT id, key, value, scope, description, is_secret, created_at, updated_at
         FROM variables WHERE scope = ?1 AND key = ?2",
        params![scope, key],
        map_variable,
    )
    .optional()
    .map_err(map_read_error)
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
    .map_err(map_read_error)?
    .ok_or(DbError::NotFound)
}

fn map_variable(row: &rusqlite::Row<'_>) -> rusqlite::Result<Variable> {
    let stored: String = row.get(2)?;
    let value = crypto::open(&stored).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(err)),
        )
    })?;
    Ok(Variable {
        id: row.get(0)?,
        key: row.get(1)?,
        value,
        scope: row.get(3)?,
        description: row.get(4)?,
        is_secret: row.get::<_, i64>(5)? != 0,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_read_error(err: rusqlite::Error) -> DbError {
    match err {
        rusqlite::Error::FromSqlConversionFailure(_, _, source) => {
            DbError::Crypto(source.to_string())
        }
        other => DbError::Other(other),
    }
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
        crypto::use_ephemeral_key();
        let path = std::env::temp_dir().join(format!(
            "key-desk-{}-{}.db",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let conn = open(&path).expect("open test db");
        (conn, path)
    }

    #[test]
    fn default_database_is_outside_the_repo() {
        if std::env::var_os("KEY_DESK_DB").is_some() {
            return;
        }
        let path = db_path();
        let text = path.to_string_lossy();
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("variables.db")
        );
        assert!(!text.contains("CARGO_MANIFEST_DIR"));
        assert!(text.contains("key-desk"), "{text}");
        #[cfg(target_os = "macos")]
        assert!(text.contains("Application Support"), "{text}");
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
        assert_eq!(created.value, "secret");
        let stored: String = conn
            .query_row(
                "SELECT value FROM variables WHERE id = ?1",
                [created.id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(stored.starts_with("kd1:"));
        assert!(!stored.contains("secret"));
        assert_eq!(list(&conn, Some("dev")).unwrap().len(), 1);
        let found = find_in_scope(&conn, "dev", "API_TOKEN").unwrap().unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.value, "secret");
        assert!(find_in_scope(&conn, "prod", "API_TOKEN").unwrap().is_none());
        assert!(matches!(insert(&conn, &input), Err(DbError::Conflict)));
        delete(&conn, created.id).unwrap();
        assert!(list(&conn, None).unwrap().is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn corrupted_ciphertext_is_reported_instead_of_exported_as_a_value() {
        let (conn, path) = temp_db();
        let input = NewVariable {
            key: "API_TOKEN".to_string(),
            value: "secret".to_string(),
            scope: "dev".to_string(),
            description: String::new(),
            is_secret: true,
        };
        let created = insert(&conn, &input).unwrap();
        conn.execute(
            "UPDATE variables SET value = 'kd1:broken' WHERE id = ?1",
            [created.id],
        )
        .unwrap();
        assert!(matches!(list(&conn, None), Err(DbError::Crypto(_))));
        assert!(matches!(
            find_in_scope(&conn, "dev", "API_TOKEN"),
            Err(DbError::Crypto(_))
        ));
        let _ = std::fs::remove_file(path);
    }
}

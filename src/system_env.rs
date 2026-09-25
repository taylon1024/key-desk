//! 环境变量快照：当前进程，或通过**登录 Shell** 拉取（更接近 Terminal 里的 `printenv`）。
//!
//! macOS 没有单一的「系统 env 表」；GUI 与 Terminal 差异主要来自是否执行 `~/.zprofile` / `~/.zshrc` 等。

use std::path::PathBuf;
use std::process::Command;

use crate::db::{self, DbError};
use crate::models::{UpsertVariable, validate_key};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnvSource {
    /// `std::env::vars()`，即 key-desk 启动时继承的环境。
    #[default]
    Process,
    /// `$SHELL -l -c printenv`，会执行登录 shell 配置。
    LoginShell,
}

impl EnvSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Process => "process",
            Self::LoginShell => "login shell",
        }
    }

    pub fn import_description(self) -> &'static str {
        match self {
            Self::Process => "imported from process env",
            Self::LoginShell => "imported from login shell",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Process => {
                "Variables inherited when key-desk started. A Dock launch usually has fewer than Terminal."
            }
            Self::LoginShell => {
                "printenv from a login shell, including ~/.zprofile. Close to a new Terminal window, not a one-off export in the current tab."
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    /// 能否通过本应用的变量名校验。
    pub importable: bool,
}

#[derive(Debug, Default)]
pub struct ImportReport {
    pub imported: usize,
    pub skipped_invalid: usize,
    pub skipped_conflict: usize,
    pub errors: Vec<String>,
}

pub fn list_env(source: EnvSource) -> Result<Vec<EnvVar>, String> {
    match source {
        EnvSource::Process => Ok(list_process_env()),
        EnvSource::LoginShell => list_login_shell_env(),
    }
}

/// 采集并按键名排序。
pub fn list_process_env() -> Vec<EnvVar> {
    from_pairs(std::env::vars())
}

/// 登录 shell 里是否已有这个变量。未设置时返回 `Ok(None)`。
pub fn lookup_login_shell_var(key: &str) -> Result<Option<String>, String> {
    let shell = login_shell();
    let shell_display = shell.display().to_string();
    let output = Command::new(&shell)
        .arg("-l")
        .arg("-c")
        .arg("/usr/bin/printenv \"$1\"")
        .arg("key-desk")
        .arg(key)
        .output()
        .map_err(|err| shell_error(&shell_display, err))?;
    if output.status.success() {
        Ok(Some(trim_printenv_newline(&output.stdout)))
    } else if output.status.code() == Some(1) {
        Ok(None)
    } else {
        Err(shell_status_error(output.status, &output.stderr))
    }
}

fn trim_printenv_newline(bytes: &[u8]) -> String {
    let mut text = String::from_utf8_lossy(bytes).into_owned();
    if text.ends_with('\n') {
        text.pop();
    }
    text
}

fn list_login_shell_env() -> Result<Vec<EnvVar>, String> {
    let shell = login_shell();
    let shell_display = shell.display().to_string();
    let output = Command::new(&shell)
        .arg("-l")
        .arg("-c")
        .arg("/usr/bin/printenv")
        .output()
        .map_err(|err| shell_error(&shell_display, err))?;
    if !output.status.success() {
        return Err(shell_status_error(output.status, &output.stderr));
    }
    Ok(parse_printenv(&String::from_utf8_lossy(&output.stdout)))
}

fn shell_error(shell: &str, err: std::io::Error) -> String {
    let message = format!("failed to run login shell ({shell}): {err}");
    log::warn!("{message}");
    message
}

fn shell_status_error(status: std::process::ExitStatus, stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let message = format!("login shell failed ({status}): {}", stderr.trim());
    log::warn!("{message}");
    message
}

fn login_shell() -> PathBuf {
    std::env::var("SHELL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/bin/zsh"))
}

fn from_pairs(pairs: impl IntoIterator<Item = (String, String)>) -> Vec<EnvVar> {
    let mut vars: Vec<EnvVar> = pairs
        .into_iter()
        .map(|(key, value)| EnvVar {
            importable: validate_key(&key).is_ok(),
            key,
            value,
        })
        .collect();
    vars.sort_by(|a, b| a.key.cmp(&b.key));
    vars
}

fn parse_printenv(text: &str) -> Vec<EnvVar> {
    let pairs = text.lines().filter_map(|line| {
        if line.is_empty() {
            return None;
        }
        let (key, value) = line.split_once('=')?;
        if key.is_empty() {
            return None;
        }
        Some((key.to_string(), value.to_string()))
    });
    from_pairs(pairs)
}

pub fn guess_is_secret(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    upper.contains("SECRET")
        || upper.contains("TOKEN")
        || upper.contains("PASSWORD")
        || upper.contains("API_KEY")
        || upper.ends_with("_KEY")
        || upper.contains("CREDENTIAL")
        || upper.contains("PRIVATE")
}

pub fn upsert_from_env(
    key: String,
    value: String,
    scope: &str,
    description: &str,
) -> UpsertVariable {
    UpsertVariable {
        is_secret: Some(guess_is_secret(&key)),
        key,
        value,
        scope: Some(scope.to_string()),
        description: Some(description.to_string()),
    }
}

/// 逐条走 `normalize` + `db::insert`；冲突与校验失败计入报告，不中断整批。
pub fn import_into_db(
    conn: &rusqlite::Connection,
    entries: impl IntoIterator<Item = (String, String)>,
    scope: &str,
    import_description: &str,
) -> ImportReport {
    let mut report = ImportReport::default();
    for (key, value) in entries {
        let upsert = upsert_from_env(key, value, scope, import_description);
        let normalized = match upsert.normalize() {
            Ok(row) => row,
            Err(err) => {
                report.skipped_invalid += 1;
                report.errors.push(err);
                continue;
            }
        };
        match db::insert(conn, &normalized) {
            Ok(_) => report.imported += 1,
            Err(DbError::Conflict) => report.skipped_conflict += 1,
            Err(err) => report.errors.push(format_db(err)),
        }
    }
    report
}

fn format_db(err: DbError) -> String {
    match err {
        DbError::NotFound => "variable not found".to_string(),
        DbError::Conflict => "name already exists in this scope".to_string(),
        DbError::Crypto(err) => format!("crypto failed: {err}"),
        DbError::Other(err) => format!("database error: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_common_secret_names() {
        assert!(guess_is_secret("OPENAI_API_KEY"));
        assert!(guess_is_secret("MY_TOKEN"));
        assert!(!guess_is_secret("PATH"));
    }

    #[test]
    fn upsert_normalizes_valid_env_key() {
        let row = upsert_from_env(
            "DATABASE_URL".into(),
            "postgres://".into(),
            "env",
            "imported from login shell",
        )
        .normalize()
        .unwrap();
        assert_eq!(row.scope, "env");
        assert_eq!(row.key, "DATABASE_URL");
    }

    #[test]
    fn missing_login_shell_var_is_absent() {
        let found = lookup_login_shell_var("KEY_DESK_MISSING_VAR_ZZZ").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn trims_only_the_trailing_newline_printenv_adds() {
        assert_eq!(trim_printenv_newline(b"sk-test\n"), "sk-test");
        assert_eq!(trim_printenv_newline(b"keep\nline"), "keep\nline");
    }

    #[test]
    fn parses_printenv_lines() {
        let vars = parse_printenv("PATH=/usr/bin\nOPENAI_API_KEY=sk-x\n");
        assert_eq!(vars.len(), 2);
        assert!(
            vars.iter()
                .any(|v| v.key == "PATH" && v.value == "/usr/bin")
        );
    }
}

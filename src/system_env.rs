//! 环境变量快照：当前进程，或一份更接近新终端的环境。
//!
//! macOS / Linux 用登录 shell 的 `printenv`（会执行 `~/.zprofile` 等）。
//! Windows 没有登录 shell，改为读取用户与系统环境变量：同名项用户覆盖系统，
//! `Path` 则按 Windows 的习惯把系统路径和用户路径拼在一起。

#[cfg(not(windows))]
use std::path::PathBuf;
use std::process::Command;

use crate::db::{self, DbError};
use crate::models::{UpsertVariable, validate_key};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnvSource {
    /// `std::env::vars()`，即 key-desk 启动时继承的环境。
    #[default]
    Process,
    /// 新终端能看到的环境：Unix 登录 shell，或 Windows 的用户 + 系统变量。
    LoginShell,
}

impl EnvSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Process => "process",
            Self::LoginShell => login_shell_label(),
        }
    }

    pub fn import_description(self) -> &'static str {
        match self {
            Self::Process => "imported from process env",
            Self::LoginShell => login_shell_import_description(),
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Process => process_hint(),
            Self::LoginShell => login_shell_hint(),
        }
    }
}

fn process_hint() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Variables inherited when key-desk started. A Dock launch usually has fewer than Terminal."
    }
    #[cfg(windows)]
    {
        "Variables inherited when key-desk started. A shortcut launch may differ from a new console."
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        "Variables inherited when key-desk started. A desktop launch may differ from a terminal."
    }
}

fn login_shell_label() -> &'static str {
    #[cfg(windows)]
    {
        "user + machine"
    }
    #[cfg(not(windows))]
    {
        "login shell"
    }
}

fn login_shell_import_description() -> &'static str {
    #[cfg(windows)]
    {
        "imported from Windows user and machine environment"
    }
    #[cfg(not(windows))]
    {
        "imported from login shell"
    }
}

fn login_shell_hint() -> &'static str {
    #[cfg(windows)]
    {
        "User and machine variables. User wins on conflict; Path is machine then user. Close to a new console."
    }
    #[cfg(not(windows))]
    {
        "printenv from a login shell, including ~/.zprofile. Close to a new Terminal window, not a one-off export in the current tab."
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

fn list_login_shell_env() -> Result<Vec<EnvVar>, String> {
    #[cfg(windows)]
    {
        list_windows_user_env()
    }
    #[cfg(not(windows))]
    {
        list_unix_login_shell_env()
    }
}

#[cfg(not(windows))]
fn list_unix_login_shell_env() -> Result<Vec<EnvVar>, String> {
    let shell = login_shell();
    let shell_display = shell.display().to_string();
    let output = Command::new(&shell)
        .arg("-l")
        .arg("-c")
        .arg("/usr/bin/printenv")
        .output()
        .map_err(|err| format!("failed to run login shell ({shell_display}): {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "login shell failed ({}): {}",
            output.status,
            stderr.trim()
        ));
    }
    Ok(parse_printenv(&String::from_utf8_lossy(&output.stdout)))
}

#[cfg(not(windows))]
fn login_shell() -> PathBuf {
    std::env::var("SHELL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/bin/zsh"))
}

/// 用户环境覆盖系统环境；`Path` 按「系统;用户」拼接，接近新控制台看到的结果。
#[cfg(windows)]
fn list_windows_user_env() -> Result<Vec<EnvVar>, String> {
    let script = r#"
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding $false
$machine = [Environment]::GetEnvironmentVariables('Machine')
$user = [Environment]::GetEnvironmentVariables('User')
$keys = @{}
foreach ($name in $machine.Keys) { $keys[$name] = [string]$machine[$name] }
foreach ($name in $user.Keys) { $keys[$name] = [string]$user[$name] }
if ($machine.Contains('Path') -and $user.Contains('Path')) {
  $machinePath = ([string]$machine['Path']).TrimEnd(';')
  $userPath = ([string]$user['Path']).TrimStart(';')
  $keys['Path'] = "$machinePath;$userPath"
}
foreach ($entry in $keys.GetEnumerator()) {
  Write-Output ('{0}={1}' -f $entry.Key, $entry.Value)
}
"#;
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    hide_console(&mut command);
    let output = command
        .output()
        .map_err(|err| format!("failed to read Windows environment: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Windows environment query failed ({}): {}",
            output.status,
            stderr.trim()
        ));
    }
    Ok(parse_printenv(&String::from_utf8_lossy(&output.stdout)))
}

#[cfg(windows)]
fn hide_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
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
        let line = line.trim_end_matches('\r');
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
    fn parses_printenv_lines() {
        let vars = parse_printenv("PATH=/usr/bin\nOPENAI_API_KEY=sk-x\n");
        assert_eq!(vars.len(), 2);
        assert!(
            vars.iter()
                .any(|v| v.key == "PATH" && v.value == "/usr/bin")
        );
    }

    #[test]
    fn parses_windows_crlf_lines() {
        let vars = parse_printenv("Path=C:\\Windows\r\nOPENAI_API_KEY=sk-x\r\n");
        assert_eq!(vars.len(), 2);
        assert!(
            vars.iter()
                .any(|v| v.key == "Path" && v.value == "C:\\Windows")
        );
    }
}

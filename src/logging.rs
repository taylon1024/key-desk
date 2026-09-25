//! 调试日志。macOS 写到 `~/Library/Logs/key-desk/`，Windows 写到 `%LOCALAPPDATA%\key-desk\logs\`。
//! 同时打到 stderr，方便 `cargo run` 时查看。不记录变量值。

use std::path::PathBuf;

pub fn init() {
    let path = log_file_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let level = level_filter();
    let dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for("key_desk", level)
        .chain(std::io::stderr());
    let dispatch = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(file) => dispatch.chain(file),
        Err(err) => {
            eprintln!("key-desk: log file unavailable ({}): {err}", path.display());
            dispatch
        }
    };
    if dispatch.apply().is_err() {
        eprintln!("key-desk: logger already initialized");
        return;
    }
    log::info!(
        "key-desk {} logging at {level} to {}",
        env!("CARGO_PKG_VERSION"),
        path.display()
    );
}

pub fn log_file_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        home_dir().join("Library/Logs/key-desk/key-desk.log")
    }
    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir().join("AppData").join("Local"))
            .join("key-desk")
            .join("logs")
            .join("key-desk.log");
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        home_dir().join(".local/state/key-desk/key-desk.log")
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

fn level_filter() -> log::LevelFilter {
    std::env::var("KEY_DESK_LOG")
        .ok()
        .and_then(|value| parse_level(&value))
        .unwrap_or(if cfg!(feature = "dev") {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
}

fn parse_level(value: &str) -> Option<log::LevelFilter> {
    Some(match value.trim().to_ascii_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" | "warning" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        "off" => log::LevelFilter::Off,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_path_matches_the_platform() {
        let path = log_file_path();
        let text = path.to_string_lossy().replace('\\', "/");
        #[cfg(target_os = "macos")]
        assert!(
            text.ends_with("Library/Logs/key-desk/key-desk.log"),
            "{text}"
        );
        #[cfg(target_os = "windows")]
        assert!(text.ends_with("key-desk/logs/key-desk.log"), "{text}");
    }

    #[test]
    fn parses_log_levels() {
        assert_eq!(parse_level("debug"), Some(log::LevelFilter::Debug));
        assert_eq!(parse_level("WARN"), Some(log::LevelFilter::Warn));
        assert_eq!(parse_level("nope"), None);
    }
}

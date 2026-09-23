//! AES 密钥的系统存储。调用方式在各平台相同：`keyring::Entry::new(服务, 账户)`。
//! `keyring` 在编译期选择后端（见 `Cargo.toml` 的 features）：
//!
//! - macOS + `apple-native`：登录钥匙串。服务名是条目的 name，账户名是 account。
//!   不指定 target 时用 User（登录）钥匙串。
//! - Windows + `windows-native`：凭据管理器里的通用凭据。目标名是
//!   `{账户}.{服务}`，即 `sqlite-value-key.key-desk`。持久级别由 keyring
//!   设为 `CRED_PERSIST_ENTERPRISE`（重启后仍在，域环境可漫游）。
//! - 其他系统：未启用原生 feature 时 keyring 使用内存 mock。磁盘上的
//!   `*.db.key` 仍是实际密钥副本，打开已有数据库不依赖这次调用。

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;

pub const SERVICE: &str = "key-desk";
pub const ACCOUNT: &str = "sqlite-value-key";

pub fn backend_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "keychain"
    }
    #[cfg(windows)]
    {
        "Credential Manager"
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        "secret store"
    }
}

pub fn load_or_create() -> Result<[u8; 32], String> {
    let entry = platform_entry()?;
    match entry.get_password() {
        Ok(stored) => decode_key(&stored),
        Err(keyring::Error::NoEntry) => {
            let key = random_key()?;
            entry
                .set_password(&encode_key(&key))
                .map_err(|err| format!("failed to store key in {}: {err}", backend_name()))?;
            Ok(key)
        }
        Err(err) => Err(format!("failed to read {}: {err}", backend_name())),
    }
}

fn platform_entry() -> Result<keyring::Entry, String> {
    #[cfg(target_os = "macos")]
    {
        keyring::Entry::new(SERVICE, ACCOUNT).map_err(|err| format!("keychain: {err}"))
    }
    #[cfg(windows)]
    {
        keyring::Entry::new(SERVICE, ACCOUNT).map_err(|err| format!("Credential Manager: {err}"))
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        keyring::Entry::new(SERVICE, ACCOUNT).map_err(|err| format!("secret store: {err}"))
    }
}

fn random_key() -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key)
        .map_err(|err| format!("failed to generate random bytes: {err}"))?;
    Ok(key)
}

fn encode_key(key: &[u8; 32]) -> String {
    B64.encode(key)
}

fn decode_key(stored: &str) -> Result<[u8; 32], String> {
    let bytes = B64
        .decode(stored.trim())
        .map_err(|_| format!("{} key has an invalid format", backend_name()))?;
    bytes
        .try_into()
        .map_err(|_| format!("{} key has the wrong length", backend_name()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_names_stay_stable() {
        assert_eq!(SERVICE, "key-desk");
        assert_eq!(ACCOUNT, "sqlite-value-key");
    }
}

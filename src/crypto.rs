//! 变量值的落盘加密：AES-256-GCM。新密钥放在系统存储里
//! （macOS 钥匙串 / Windows 凭据管理器）。旧版同名 `*.db.key` 仍会读取，不再新建。
//!
//! 数据库里存 `kd1:` + base64(nonce || ciphertext)。没有此前缀的旧数据视为明文，打开库时会改写成密文。
//! 密文格式在各平台相同。已有的 `*.db.key` 继续有效，迁移完成前不要删除。

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;

const PREFIX: &str = "kd1:";

static KEY_OVERRIDE: Mutex<Option<[u8; 32]>> = Mutex::new(None);
static KEY_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// 只为兼容旧数据库保留同名 `.key` 文件的读取路径；新密钥留在系统存储。
pub fn bind_store(db_path: &Path) {
    let key_path = PathBuf::from(format!("{}.key", db_path.display()));
    *KEY_FILE.lock().expect("key file") = Some(key_path);
}

/// 测试用固定密钥，避免碰系统钥匙串。
#[cfg(test)]
pub fn use_ephemeral_key() {
    let mut key = [7u8; 32];
    key[0] = 0x6b;
    *KEY_OVERRIDE.lock().expect("key override") = Some(key);
}

pub fn is_sealed(stored: &str) -> bool {
    stored.starts_with(PREFIX)
}

pub fn seal(plaintext: &str) -> Result<String, String> {
    let key = data_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|err| err.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    fill_random(&mut nonce_bytes)?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| "encryption failed".to_string())?;
    let mut packed = Vec::with_capacity(12 + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);
    Ok(format!("{PREFIX}{}", B64.encode(packed)))
}

pub fn open(stored: &str) -> Result<String, String> {
    let Some(payload) = stored.strip_prefix(PREFIX) else {
        return Ok(stored.to_string());
    };
    let packed = B64
        .decode(payload.trim())
        .map_err(|_| "invalid ciphertext".to_string())?;
    if packed.len() < 12 + 16 {
        return Err("ciphertext is too short".to_string());
    }
    let (nonce_bytes, ciphertext) = packed.split_at(12);
    let key = data_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|err| err.to_string())?;
    let plain = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| {
            format!(
                "decryption failed; the {} key may not match this database",
                crate::os_key::backend_name()
            )
        })?;
    String::from_utf8(plain).map_err(|_| "decrypted value is not text".to_string())
}

fn data_key() -> Result<[u8; 32], String> {
    if let Some(key) = *KEY_OVERRIDE.lock().expect("key override") {
        return Ok(key);
    }
    if let Some(path) = KEY_FILE.lock().expect("key file").clone()
        && path.exists()
    {
        return read_key_file(&path);
    }
    crate::os_key::load_or_create()
}

fn read_key_file(path: &Path) -> Result<[u8; 32], String> {
    let stored = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
    decode_key(&stored)
}

fn fill_random(buf: &mut [u8]) -> Result<(), String> {
    getrandom::getrandom(buf).map_err(|err| format!("failed to generate random bytes: {err}"))
}

fn decode_key(stored: &str) -> Result<[u8; 32], String> {
    let bytes = B64
        .decode(stored.trim())
        .map_err(|_| "key file has an invalid format".to_string())?;
    bytes
        .try_into()
        .map_err(|_| "key file has the wrong length".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_leaves_plaintext_alone() {
        use_ephemeral_key();
        let sealed = seal("sk-live").unwrap();
        assert!(is_sealed(&sealed));
        assert_ne!(sealed, "sk-live");
        assert_eq!(open(&sealed).unwrap(), "sk-live");
        assert_eq!(open("still-plain").unwrap(), "still-plain");
    }
}

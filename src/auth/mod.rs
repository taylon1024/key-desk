//! 打开数据库前的本机用户确认。
//!
//! - macOS：Touch ID，失败或没有指纹时回退到账户密码（LocalAuthentication）。
//! - Windows：不调用 Windows Hello。当前登录会话就是边界；AES 密钥放在凭据管理器里，读取时不再弹窗。
//! - 其他系统：同样立即解锁，密钥由 `keyring` 在该平台上的默认存储处理。
//!
//! 开发分支默认启用 `dev` feature，窗口会直接打开库，不走这里的确认。

use std::sync::mpsc::Receiver;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", windows)))]
mod other;
#[cfg(windows)]
mod windows;

pub struct PendingAuth {
    #[cfg_attr(feature = "dev", allow(dead_code))]
    pub rx: Receiver<Result<(), String>>,
    #[cfg(target_os = "macos")]
    _macos: macos::Session,
}

pub fn begin(on_done: impl Fn() + Send + 'static) -> Result<PendingAuth, String> {
    #[cfg(target_os = "macos")]
    {
        macos::begin(on_done)
    }
    #[cfg(windows)]
    {
        windows::begin(on_done)
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        other::begin(on_done)
    }
}

pub fn lock_prompt() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Touch ID or Mac password"
    }
    #[cfg(windows)]
    {
        "Windows account session"
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        "system secret store"
    }
}

pub fn lock_detail() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "required every time key-desk opens"
    }
    #[cfg(windows)]
    {
        "Credential Manager holds the key; Windows Hello is not prompted"
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        "no biometric prompt on this platform"
    }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn unlock_immediately(on_done: impl Fn() + Send + 'static) -> PendingAuth {
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = tx.send(Ok(()));
    on_done();
    PendingAuth { rx }
}

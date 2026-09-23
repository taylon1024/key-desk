//! Windows 解锁。
//!
//! 不接 Windows Hello：`windows` crate 的生物识别流程要单独处理异步回调和
//! 用户取消，而凭据管理器本身按当前用户保护密钥，读取时也不会再弹系统对话框。
//! 因此这里在窗口起来后立即解锁，真正的密钥读写发生在 `os_key`。

use super::{PendingAuth, unlock_immediately};

pub fn begin(on_done: impl Fn() + Send + 'static) -> Result<PendingAuth, String> {
    Ok(unlock_immediately(on_done))
}

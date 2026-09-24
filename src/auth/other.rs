//! 非 macOS / Windows：没有接入的生物识别 API，直接解锁。

use super::{PendingAuth, unlock_immediately};

pub fn begin(on_done: impl Fn() + Send + 'static) -> Result<PendingAuth, String> {
    Ok(unlock_immediately(on_done))
}

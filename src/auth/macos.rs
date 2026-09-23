//! macOS owner authentication: Touch ID, with the account password as fallback.
//! The prompt is asynchronous so the window run loop can keep drawing it.

use std::sync::mpsc;

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_foundation::{NSError, NSString};
use objc2_local_authentication::{LAContext, LAError, LAPolicy};

use super::PendingAuth;

pub struct Session {
    _context: objc2::rc::Retained<LAContext>,
    _block: RcBlock<dyn Fn(Bool, *mut NSError)>,
}

pub fn begin(on_done: impl Fn() + Send + 'static) -> Result<PendingAuth, String> {
    let context = unsafe { LAContext::new() };
    let policy = LAPolicy::DeviceOwnerAuthentication;
    if let Err(err) = unsafe { context.canEvaluatePolicy_error(policy) } {
        return Err(ns_error_text(&err));
    }

    let (tx, rx) = mpsc::channel();
    let block = RcBlock::new(move |success: Bool, error: *mut NSError| {
        let result = if success.as_bool() {
            Ok(())
        } else {
            Err(error_from_ptr(error))
        };
        let _ = tx.send(result);
        on_done();
    });
    let reason = NSString::from_str("unlock key-desk");
    unsafe {
        context.evaluatePolicy_localizedReason_reply(policy, &reason, &block);
    }
    Ok(PendingAuth {
        rx,
        _macos: Session {
            _context: context,
            _block: block,
        },
    })
}

fn error_from_ptr(error: *mut NSError) -> String {
    if error.is_null() {
        "authentication failed".to_string()
    } else {
        ns_error_text(unsafe { &*error })
    }
}

fn ns_error_text(error: &NSError) -> String {
    match LAError(error.code()) {
        LAError::UserCancel | LAError::SystemCancel | LAError::AppCancel => "cancelled".to_string(),
        LAError::PasscodeNotSet => "set a Mac password before key-desk can lock".to_string(),
        _ => error.localizedDescription().to_string(),
    }
}

//! Thin re-export shim so the rest of the crate can keep writing
//! `logger::log_message(...)` etc. while delegating to rcommon.

#[allow(unused_imports)]
pub use library::lifecycle::background::file_log::{
    get_appdata_log_path, is_event_log_enabled, log_message, set_event_log_enabled,
    set_event_source, set_log_app_name,
};

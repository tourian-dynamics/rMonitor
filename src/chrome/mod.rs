//! Chrome helper layer.

pub mod embedded_docs;
pub mod app_state;

pub use embedded_docs::{doc, open_embedded_markdown};
pub use app_state::scroll_for_key;

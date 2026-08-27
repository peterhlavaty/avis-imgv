//! Things the user can trigger on an image: external commands bound to
//! shortcuts or context menu entries, and the callbacks that run afterwards.

pub mod callback;
pub mod user_action;

pub use callback::Callback;
pub use user_action::{execute, show_context_menu};

//! Things the user can trigger on an image: external commands bound to
//! shortcuts or context menu entries, and the callbacks that run afterwards.

pub mod callback;
pub mod reveal;
// The one file in this directory that draws: the menu a user action appears
// in. Gated for the same reason `history::panel` is.
#[cfg(feature = "gui")]
mod user_action;

pub use callback::Callback;
#[cfg(feature = "gui")]
pub use user_action::{execute, show_context_menu};

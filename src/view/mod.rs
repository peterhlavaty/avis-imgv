//! The two ways of looking at a folder: one image at a time, or all of them at
//! once.

pub mod grid_view;
pub mod image_view;
pub mod organize;
pub mod texture;
pub mod wheel;

pub use grid_view::GridView;
pub use image_view::ImageView;

// What the views are *about* rather than how they draw it: which photographs
// are open, in what order, folded how, and which are picked out. It lives in
// `crate::collection` now — none of it needs a window, and `config` and
// `history` both read it — and is named here because the two views ask for it
// constantly and this is the door they already come to.
pub use crate::collection::{
    narrow::Narrowing, selection::Selection, stacks::Stacks, visible::Visible,
};

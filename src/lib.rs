//! avis-imgv: a GPU accelerated image viewer.
//!
//! The design is one idea applied consistently: get the whole folder decoded
//! into RAM on background threads, keep the images around the cursor resident
//! on the GPU, and let drawing be nothing but a textured quad.
//!
//! - [`atomic`] puts a file in place in one step,
//! - [`board`] is where a frame leaves a value for the rest of itself,
//! - [`fault`] is how something that went wrong says so, once,
//! - [`work`] is a background queue that knows nothing about photographs,
//! - [`fit`] puts one rectangle inside another, keeping its shape,
//! - [`crawler`] finds the images,
//! - [`decoder`] turns bytes into RGBA8, reading [`metadata`] from the same
//!   buffer instead of shelling out,
//! - [`cache`] decides what lives in RAM and what lives on the GPU,
//! - [`view`] draws it, [`ui`] is the chrome round it — the cards, the
//!   panels and the menus — and [`app`] wires it all together,
//! - [`organize`] works on the folder rather than the image: bulk renaming,
//!   correcting a camera clock, and the bin the viewer keeps of its own,
//! - [`history`] remembers what was done and how to get back to any of it,
//! - [`annotations`] holds what the user puts on an image: stars and tags,
//!   kept in XMP sidecars.

pub mod actions;
pub mod annotations;
pub mod app;
pub mod atomic;
pub mod board;
pub mod cache;
pub mod config;
pub mod crawler;
pub mod decoder;
pub mod fault;
pub mod fit;
pub mod formats;
pub mod history;
pub mod logging;
pub mod metadata;
pub mod organize;
pub mod session;
pub mod ui;
pub mod utils;
pub mod view;
pub mod work;

/// Identifiers used to locate the per-user configuration directory.
pub const QUALIFIER: &str = "com";
pub const ORGANIZATION: &str = "avis-imgv";
pub const APPLICATION: &str = "avis-imgv";

/// Command line flags that change the starting state rather than naming a path.
pub const STARTER_STATE_ARGS: &[&str] = &[
    "--slideshow",
    "--fullscreen",
    "--benchmark",
    "--reset-text-size",
];

//! avis-imgv: a GPU accelerated image viewer.
//!
//! The design is one idea applied consistently: get the whole folder decoded
//! into RAM on background threads, keep the images around the cursor resident
//! on the GPU, and let drawing be nothing but a textured quad.
//!
//! - [`atomic`] puts a file in place in one step,
//! - [`crawler`] finds the images,
//! - [`decoder`] turns bytes into RGBA8, reading [`metadata`] from the same
//!   buffer instead of shelling out,
//! - [`cache`] decides what lives in RAM and what lives on the GPU,
//! - [`view`] draws it, [`app`] wires it together,
//! - [`organize`] works on the folder rather than the image: bulk renaming
//!   and correcting a camera clock,
//! - [`annotations`] holds what the user puts on an image: stars and tags,
//!   kept in XMP sidecars.

pub mod actions;
pub mod annotations;
pub mod app;
pub mod atomic;
pub mod cache;
pub mod config;
pub mod crawler;
pub mod decoder;
pub mod formats;
pub mod logging;
pub mod metadata;
pub mod organize;
pub mod session;
pub mod ui;
pub mod utils;
pub mod view;

/// Identifiers used to locate the per-user configuration directory.
pub const QUALIFIER: &str = "com";
pub const ORGANIZATION: &str = "avis-imgv";
pub const APPLICATION: &str = "avis-imgv";

/// Command line flags that change the starting state rather than naming a path.
pub const STARTER_STATE_ARGS: &[&str] = &["--slideshow", "--fullscreen", "--benchmark"];

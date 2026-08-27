//! avis-imgv: a GPU accelerated image viewer.
//!
//! The design is one idea applied consistently: get the whole folder decoded
//! into RAM on background threads, keep the images around the cursor resident
//! on the GPU, and let drawing be nothing but a textured quad.
//!
//! - [`crawler`] finds the images,
//! - [`decoder`] turns bytes into RGBA8, reading [`metadata`] from the same
//!   buffer instead of shelling out,
//! - [`cache`] decides what lives in RAM and what lives on the GPU,
//! - [`view`] draws it, [`app`] wires it together.

pub mod actions;
pub mod app;
pub mod cache;
pub mod config;
pub mod crawler;
pub mod decoder;
pub mod formats;
pub mod metadata;
pub mod ui;
pub mod utils;
pub mod view;

/// Identifiers used to locate the per-user configuration directory.
pub const QUALIFIER: &str = "com";
pub const ORGANIZATION: &str = "avis-imgv";
pub const APPLICATION: &str = "avis-imgv";

/// Command line flags that change the starting state rather than naming a path.
pub const STARTER_STATE_ARGS: &[&str] = &["--slideshow", "--fullscreen"];

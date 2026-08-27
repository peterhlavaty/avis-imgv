//! Default values for every configurable field.
//!
//! Kept apart from the structs so serde's `default = "..."` attributes stay
//! readable and the shape of the configuration is visible at a glance.

use super::shortcut::{Shortcut, MOD_ALT, MOD_CTRL};
use super::{ContextMenuEntry, RawQuality, RawSource, TagCategory, UserAction};

/// Four gigabytes of decoded pixels: generous on a modern machine and still
/// small enough not to push a 16 GB laptop into swap.
pub fn default_ram_budget_mb() -> usize {
    4096
}

/// Zero means one worker per core, less one left for the UI thread.
pub fn default_decode_threads() -> usize {
    0
}

pub fn default_uploads_per_frame() -> usize {
    4
}

pub fn default_output_icc_profile() -> String {
    String::from("srgb")
}

pub fn default_text_scaling() -> f32 {
    1.25
}

pub fn default_sc_toggle_gallery() -> Shortcut {
    Shortcut::new("Backspace", &[])
}

pub fn default_sc_exit() -> Shortcut {
    Shortcut::new("q", &[MOD_ALT])
}

pub fn default_sc_menu() -> Shortcut {
    Shortcut::new("F1", &[])
}

pub fn default_sc_navigator() -> Shortcut {
    Shortcut::new("l", &[MOD_CTRL])
}

pub fn default_sc_dir_tree() -> Shortcut {
    Shortcut::new("t", &[])
}

pub fn default_sc_flatten_dir() -> Shortcut {
    Shortcut::new("f", &[MOD_CTRL])
}

pub fn default_sc_watch_directory() -> Shortcut {
    Shortcut::new("w", &[MOD_CTRL])
}

//Image view
pub fn default_nr_loaded_images() -> usize {
    64
}
pub fn default_gpu_resident_images() -> usize {
    8
}
/// Zero means "as large as the GPU allows".
pub fn default_max_image_edge() -> u32 {
    0
}
pub fn default_should_wait() -> bool {
    true
}
pub fn default_metadata_tags() -> Vec<String> {
    [
        "File Name",
        "Date/Time Original",
        "Camera Model Name",
        "Lens Model",
        "Focal Length",
        "Aperture",
        "Shutter Speed",
        "ISO",
        "Image Size",
        "File Size",
        "Color Space",
        "Directory",
    ]
    .iter()
    .map(|tag| tag.to_string())
    .collect()
}
pub fn default_frame_size_relative_to_image() -> f32 {
    0.2
}
pub fn default_scroll_navigation() -> bool {
    true
}
pub fn default_name_format() -> String {
    "$(#File Name#)$( • ƒ#Aperture#)$( • #Shutter Speed#)$( • #ISO# ISO)".to_string()
}
pub fn default_user_actions() -> Vec<UserAction> {
    vec![]
}
pub fn default_ctx_menu() -> Vec<ContextMenuEntry> {
    vec![]
}
pub fn default_sc_fit() -> Shortcut {
    Shortcut::new("f", &[])
}
pub fn default_sc_frame() -> Shortcut {
    Shortcut::new("g", &[])
}
pub fn default_sc_toggle_side_panel() -> Shortcut {
    Shortcut::new("i", &[])
}
pub fn default_sc_zoom() -> Shortcut {
    Shortcut::new("Space", &[])
}
pub fn default_sc_next() -> Shortcut {
    Shortcut::new("ArrowRight", &[])
}
pub fn default_sc_prev() -> Shortcut {
    Shortcut::new("ArrowLeft", &[])
}
pub fn default_sc_one_to_one() -> Shortcut {
    Shortcut::new("1", &[MOD_ALT])
}
pub fn default_sc_fit_vertical() -> Shortcut {
    Shortcut::new("v", &[])
}
pub fn default_sc_fit_horizontal() -> Shortcut {
    Shortcut::new("h", &[])
}
pub fn default_sc_fit_maximize() -> Shortcut {
    Shortcut::new("m", &[])
}
pub fn default_sc_latch_fit_maximize() -> Shortcut {
    Shortcut::new("m", &[MOD_CTRL])
}

pub fn default_nr_images_shown() -> usize {
    1
}
pub fn default_sc_more_images_shown() -> Shortcut {
    Shortcut::new("Plus", &[])
}
pub fn default_sc_less_images_shown() -> Shortcut {
    Shortcut::new("Minus", &[])
}

//Multi Gallery
pub fn default_images_per_row() -> usize {
    5
}
pub fn default_preloaded_rows() -> usize {
    1
}
pub fn default_thumbnail_resolution() -> u32 {
    512
}
pub fn default_gpu_resident_thumbnails() -> usize {
    256
}
pub fn default_sc_scroll() -> Shortcut {
    Shortcut::new("Space", &[])
}
pub fn default_sc_more_per_row() -> Shortcut {
    Shortcut::new("Plus", &[])
}
pub fn default_sc_less_per_row() -> Shortcut {
    Shortcut::new("Minus", &[])
}

//Tags
/// A starting point that shows what categories are for without presuming what
/// anyone photographs.
pub fn default_tag_categories() -> Vec<TagCategory> {
    vec![
        TagCategory {
            name: "Status".to_string(),
            tags: ["Keeper", "Portfolio", "To edit", "Sent"]
                .iter()
                .map(|tag| tag.to_string())
                .collect(),
        },
        TagCategory {
            name: "Subject".to_string(),
            tags: ["Portrait", "Landscape", "Macro", "Wildlife", "Architecture"]
                .iter()
                .map(|tag| tag.to_string())
                .collect(),
        },
    ]
}

pub fn default_recent_tags() -> usize {
    12
}

pub fn default_tag_panel_width() -> f32 {
    260.
}

pub fn default_sc_toggle_tag_panel() -> Shortcut {
    Shortcut::new("k", &[])
}

/// The digit keys, so a rating is one keystroke away. Index 0 clears it.
pub fn default_sc_rating() -> Vec<Shortcut> {
    (0..=5)
        .map(|stars| Shortcut::new(&stars.to_string(), &[]))
        .collect()
}

//Raw
/// The preview, because it is what makes opening a folder of raws instant.
/// Developing is a deliberate choice.
pub fn default_raw_source() -> RawSource {
    RawSource::Preview
}

pub fn default_raw_quality() -> RawQuality {
    RawQuality::Balanced
}

pub fn default_camera_white_balance() -> bool {
    true
}

pub fn default_auto_brighten() -> bool {
    true
}

pub fn default_highlight_mode() -> u8 {
    0
}

//Slideshow
pub fn default_seconds_per_image() -> u64 {
    15
}

pub fn default_percent_zoom() -> f32 {
    25.
}

pub fn default_start_with_frame_enabled() -> bool {
    false
}

pub fn default_image_frame_background_color_override() -> Option<String> {
    None
}

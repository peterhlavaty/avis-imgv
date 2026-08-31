//! Default values for every configurable field.
//!
//! Kept apart from the structs so serde's `default = "..."` attributes stay
//! readable and the shape of the configuration is visible at a glance.

use super::shortcut::{Shortcut, MOD_ALT, MOD_CTRL, MOD_SHIFT};
use super::{ContextMenuEntry, Destination, RawQuality, RawSource, TagCategory, UserAction};

/// Four gigabytes of decoded pixels: generous on a modern machine and still
/// small enough not to push a 16 GB laptop into swap.
pub fn default_ram_budget_mb() -> usize {
    4096
}

/// Zero means one worker per core, less one left for the UI thread.
pub fn default_decode_threads() -> usize {
    0
}

/// A thumbnail only has to cover the moment between an image being asked for
/// and its decode landing, so a narrow window around the cursor is enough.
/// Reading further ahead would take disk bandwidth from the decoders.
pub fn default_previews_resident() -> usize {
    16
}

/// The image on screen and the two a single key press away.
///
/// Each of these is a full sized decode held in memory, so this is a number to
/// raise only if there is memory going spare.
pub fn default_full_resolution_neighbours() -> usize {
    1
}

/// Half a frame at sixty a second, which leaves the rest for drawing.
/// A gigabyte, which every adapter this viewer runs on has and which holds a
/// generous window of full size photographs.
pub fn default_gpu_budget_mb() -> usize {
    1024
}
pub fn default_upload_budget_ms() -> u64 {
    8
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

/// Not Tab, however much it looks like the key for this: egui gives Tab to
/// whatever widget is next in line, and a mode with a text box in it would
/// swallow the key on the way past.
pub fn default_sc_next_mode() -> Shortcut {
    Shortcut::new("F2", &[])
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

/// Deliberately more than any budget will grant: what is actually held is
/// trimmed to what fits, so this only has to be large enough not to be the
/// limit itself. A screen sized copy of a 24 megapixel photograph is eleven
/// megabytes, so a four gigabyte budget reaches a couple of hundred images —
/// most folders, entirely resident.
pub fn default_nr_loaded_images() -> usize {
    512
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
        // Only there when what is on screen is smaller than the photograph,
        // which for a raw file shown through its embedded copy it often is.
        "Preview Size",
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
/// `r` for repeat: the zoom and position of the image just left, applied to
/// this one, which is how two frames of the same thing get compared.
pub fn default_sc_repeat_place() -> Shortcut {
    Shortcut::new("r", &[])
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
/// Plus and minus belong to the zoom, so showing more images side by side
/// moved onto the same keys with control held.
pub fn default_sc_more_images_shown() -> Shortcut {
    Shortcut::new("Plus", &[MOD_CTRL])
}
pub fn default_sc_less_images_shown() -> Shortcut {
    Shortcut::new("Minus", &[MOD_CTRL])
}
pub fn default_sc_zoom_in() -> Shortcut {
    Shortcut::new("Plus", &[])
}
pub fn default_sc_zoom_out() -> Shortcut {
    Shortcut::new("Minus", &[])
}

/// Lightroom's Survey key, which is what everybody who has used one expects.
pub fn default_sc_compare() -> Shortcut {
    Shortcut::new("n", &[])
}

/// The four keys under the left hand, which is where they are wanted: the
/// right hand is on the mouse or the arrow keys.
pub fn default_sc_pan_up() -> Shortcut {
    Shortcut::new("w", &[])
}
pub fn default_sc_pan_down() -> Shortcut {
    Shortcut::new("s", &[])
}
pub fn default_sc_pan_left() -> Shortcut {
    Shortcut::new("a", &[])
}
pub fn default_sc_pan_right() -> Shortcut {
    Shortcut::new("d", &[])
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
/// `PageDown` rather than the space bar it used to be.
///
/// Space is what every program with a contact sheet uses to pick a photograph
/// out, and picking photographs out is worth more than a key for scrolling
/// half a row when the arrows, the wheel and the scrollbar all already do it.
pub fn default_sc_scroll() -> Shortcut {
    Shortcut::new("PageDown", &[])
}

/// Bridge's key, and Photo Mechanic's, and every file manager's.
pub fn default_sc_select() -> Shortcut {
    Shortcut::new("Space", &[])
}
pub fn default_sc_select_all() -> Shortcut {
    Shortcut::new("a", &[MOD_CTRL])
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

/// Three to two, which is what most cameras shoot and therefore what wastes
/// the least of a contact sheet.
pub fn default_cell_aspect() -> f32 {
    1.5
}

/// The same key as the overlay on the image itself, so one thing is one key.
pub fn default_sc_cycle_badges() -> Shortcut {
    Shortcut::new("i", &[MOD_CTRL])
}

/// On, because the alternative is a postage stamp and nobody asked for one.
/// A photograph larger than the window is unaffected either way.
pub fn default_enlarge_to_fit() -> bool {
    true
}

//Cull
/// Two that describe what nearly everybody does with a shoot, named
/// relatively so they follow the folder rather than pointing at one.
pub fn default_destinations() -> Vec<Destination> {
    vec![
        Destination {
            label: "Selects".to_string(),
            path: "Selects".to_string(),
        },
        Destination {
            label: "To edit".to_string(),
            path: "To edit".to_string(),
        },
    ]
}

/// FastRawViewer's name for it, which is the one people already have folders
/// called.
pub fn default_rejected_folder() -> String {
    "_Rejected".to_string()
}

/// Alt, because bare M and C are the fill-the-window and the context menu.
pub fn default_sc_move() -> Shortcut {
    Shortcut::new("m", &[MOD_ALT])
}
pub fn default_sc_copy() -> Shortcut {
    Shortcut::new("c", &[MOD_ALT])
}
/// The reject key with shift: the same verb, carried out on the disk.
pub fn default_sc_reject_folder() -> Shortcut {
    Shortcut::new("x", &[MOD_SHIFT])
}
pub fn default_sc_undo() -> Shortcut {
    Shortcut::new("z", &[MOD_CTRL])
}

/// `F3` is what a Windows program uses for "find", and the backslash is what
/// FastRawViewer uses for the bypass.
pub fn default_sc_filter() -> Shortcut {
    Shortcut::new("F3", &[])
}
pub fn default_sc_suspend_filter() -> Shortcut {
    Shortcut::new("Backslash", &[])
}

/// What every program that has a fullscreen uses.
pub fn default_sc_fullscreen() -> Shortcut {
    Shortcut::new("F11", &[])
}

/// What every file manager and every viewer in the comparison uses.
pub fn default_sc_delete() -> Shortcut {
    Shortcut::new("delete", &[])
}
pub fn default_sc_delete_permanently() -> Shortcut {
    Shortcut::new("delete", &[MOD_SHIFT])
}

/// Lightroom's keys, which every other program copied.
pub fn default_sc_pick() -> Shortcut {
    Shortcut::new("p", &[])
}
pub fn default_sc_reject() -> Shortcut {
    Shortcut::new("x", &[])
}
pub fn default_sc_unflag() -> Shortcut {
    Shortcut::new("u", &[])
}

/// The digits above the ratings, in the order the labels are listed. Purple
/// takes control because the row runs out at nine.
pub fn default_sc_label() -> Vec<Shortcut> {
    vec![
        Shortcut::new("6", &[]),
        Shortcut::new("7", &[]),
        Shortcut::new("8", &[]),
        Shortcut::new("9", &[]),
        Shortcut::new("9", &[MOD_CTRL]),
    ]
}

/// Off, because moving by itself is a surprise the first time it happens.
pub fn default_advance_after_marking() -> bool {
    false
}

/// A mode rather than a modifier: on the layouts where the digits are the
/// shifted characters of the top row, a modifier could not be told apart from
/// the rating key itself.
pub fn default_sc_toggle_advance() -> Shortcut {
    Shortcut::new("a", &[MOD_CTRL, MOD_SHIFT])
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

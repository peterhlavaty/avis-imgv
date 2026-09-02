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
pub fn default_restore_session() -> bool {
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
/// What the overlay says when it is turned on.
///
/// Two lines: what it is, and how it was taken. Each part disappears with its
/// separator when the photograph cannot answer it.
pub fn default_overlay_format() -> String {
    "{name}
$({iso} ISO)$( • ƒ{aperture})$( • {shutter})$( • {focal})"
        .to_string()
}
pub fn default_overlay_text_size() -> f32 {
    15.0
}
/// `o` for overlay, which nothing else wanted.
pub fn default_sc_overlay() -> Shortcut {
    Shortcut::new("o", &[])
}
/// `c` for clipping, which is the first thing it shows.
pub fn default_sc_marks() -> Shortcut {
    Shortcut::new("c", &[])
}
/// Enter, which is what a crop is accepted with everywhere.
pub fn default_sc_zoom_to_area() -> Shortcut {
    Shortcut::new("Enter", &[])
}
/// Enough that the eye goes to what was marked, not so much that what is
/// around it can no longer be judged against it.
pub fn default_marked_area_dim() -> u8 {
    45
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
/// `Ctrl + T` for the strip, `t` alone being the directory tree.
pub fn default_sc_filmstrip() -> Shortcut {
    Shortcut::new("t", &[MOD_CTRL])
}
/// `Ctrl + G` for grouping, which is what Lightroom and Bridge both use for
/// making a stack.
/// The two keys that turn a photograph.
///
/// `[` and `]`, which is what Lightroom and Photoshop use and what a
/// photographer's hands already know. Both are bindings rather than bare keys,
/// which matters more here than usual: on the layouts where a bracket needs
/// AltGr they are exactly the sort of key that has to be moved.
pub fn default_sc_turn_left() -> Shortcut {
    Shortcut::new("OpenBracket", &[])
}

pub fn default_sc_turn_right() -> Shortcut {
    Shortcut::new("CloseBracket", &[])
}

pub fn default_sc_stacks() -> Shortcut {
    Shortcut::new("g", &[MOD_CTRL])
}

/// `E` for expand. Not `S`, which every other program uses for this and which
/// this one has had on panning down since long before there were stacks.
pub fn default_sc_toggle_stack() -> Shortcut {
    Shortcut::new("e", &[])
}

/// The two keys either side of `M`, which is where a hand on the keys already
/// is, and which nothing else wanted.
pub fn default_sc_standing_back() -> Shortcut {
    Shortcut::new("Comma", &[])
}
pub fn default_sc_standing_forward() -> Shortcut {
    Shortcut::new("Period", &[])
}

/// The arrows step frame to frame, so the arrows with a modifier step run to
/// run.
pub fn default_sc_previous_stack() -> Shortcut {
    Shortcut::new("ArrowLeft", &[MOD_CTRL])
}
pub fn default_sc_next_stack() -> Shortcut {
    Shortcut::new("ArrowRight", &[MOD_CTRL])
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
/// A hundred per cent: one screen pixel to one of the photograph's own, which
/// is the magnification focus is judged at and the reason anybody asks for a
/// number here at all.
pub fn default_opening_percent() -> f32 {
    100.0
}

/// `Ctrl + M`, which is where the key that latched the filling already was.
/// It now moves what a photograph opens at round the three answers.
pub fn default_sc_cycle_opening() -> Shortcut {
    Shortcut::new("m", &[MOD_CTRL])
}

/// Beside `r`, which repeats the last view once: these are the same thing
/// held down, so they are the same key with a modifier.
pub fn default_sc_keep_zoom() -> Shortcut {
    Shortcut::new("r", &[MOD_CTRL])
}

pub fn default_sc_keep_pan() -> Shortcut {
    Shortcut::new("r", &[MOD_CTRL, MOD_SHIFT])
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
/// The key that drops a pane from a comparison.
///
/// Still `/` by default, because that is what it has always been and a
/// migration would be moving somebody's key for no gain. What changes is that
/// it is now a row like any other, so a layout where `/` is Shift and a digit
/// has somewhere to say so.
pub fn default_sc_drop_pane() -> Shortcut {
    Shortcut::new("Slash", &[])
}

/// The key that puts the cursor in the "go to" box.
///
/// `Ctrl + J` for "jump to". `Ctrl + G` is what an editor uses for this and it
/// is taken here by *Stacks* — which is how this default was found: the row
/// claimed to be read only in the contact sheet, so the clash checker looked
/// straight past it while the key itself was being eaten in every mode. The
/// six stack rows say `Everywhere` now, which is where they are read.
pub fn default_sc_go_to() -> Shortcut {
    Shortcut::new("J", &[MOD_CTRL])
}

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
/// Off. The strip takes room from the photograph, so it is asked for.
pub fn default_filmstrip_height() -> f32 {
    0.0
}

/// The file name, which is what the sheet has always shown.
pub fn default_caption_format() -> String {
    "{name}.{ext}".to_string()
}
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

/// The platform's, because Delete meaning what it means in every other
/// program is what nearly everybody expects the first time they press it.
pub fn default_bin() -> String {
    "system".to_string()
}

/// Nothing, meaning the folder the viewer keeps under the local data
/// directory.
///
/// A path written here rather than left out would be a path from whichever
/// machine the file was first written on, and a configuration that follows
/// somebody onto a second one would point its bin at a drive letter that is
/// not there.
pub fn default_bin_folder() -> Option<String> {
    None
}

/// Asked, because a bin left full is a bin nobody looks in and the whole point
/// of a folder is that it can be looked in.
pub fn default_ask_to_empty_the_bin() -> bool {
    true
}

/// Alt, because bare M and C are the fill-the-window and the context menu.
pub fn default_sc_move() -> Shortcut {
    Shortcut::new("m", &[MOD_ALT])
}
pub fn default_sc_copy() -> Shortcut {
    Shortcut::new("c", &[MOD_ALT])
}
/// B for back, and free: the bare letter is nothing and no other command has
/// taken it with a modifier.
pub fn default_sc_put_back() -> Shortcut {
    Shortcut::new("b", &[MOD_CTRL])
}

/// The reject key with shift: the same verb, carried out on the disk.
pub fn default_sc_reject_folder() -> Shortcut {
    Shortcut::new("x", &[MOD_SHIFT])
}
/// The two everybody already has in their fingers.
///
/// `Ctrl + Y` for redo rather than `Ctrl + Shift + Z` because this is a
/// Windows-first program by its user base, and Y is one key rather than two.
pub fn default_sc_undo() -> Shortcut {
    Shortcut::new("z", &[MOD_CTRL])
}

pub fn default_sc_redo() -> Shortcut {
    Shortcut::new("y", &[MOD_CTRL])
}

/// `Ctrl + H` for the list of what was done. H for history, and the only
/// letter of the word not already spoken for.
pub fn default_sc_history() -> Shortcut {
    Shortcut::new("h", &[MOD_CTRL])
}

/// Wide enough for a sentence about what was done without wrapping every row.
pub fn default_history_panel_width() -> f32 {
    260.0
}

/// Half a second, which is longer than a key repeats and shorter than a pause
/// for thought. Two notches of a wheel are one row; a walk taken up again
/// after looking at something is a new one.
pub fn default_merge_within_ms() -> u64 {
    500
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

/// Which mode a launch starts in. The image view, which is what a viewer is.
pub fn default_start_in() -> String {
    "image".to_string()
}

/// The metadata panel's starting width, in points.
///
/// A hardcoded `default_width(340.)` until now, which meant dragging the edge
/// was a gesture the viewer forgot on the way out.
pub fn default_side_panel_width() -> f32 {
    340.0
}

/// Dark, which is what the viewer has always been.
///
/// Not because a light interface is wrong, but because the reason the theme was
/// hardcoded — that a light surround shifts how the photograph reads — is
/// really about the backdrop, and the backdrop is its own field now.
pub fn default_theme() -> String {
    "dark".to_string()
}

/// The grey behind the photograph.
///
/// Neutral enough not to shift how a photograph reads against it, which is the
/// whole reason it is a middle grey and not the theme's own background.
pub fn default_backdrop() -> String {
    "#777777".to_string()
}

/// How much one press of the zoom keys changes the magnification.
pub fn default_zoom_step() -> f32 {
    1.25
}

/// How much the step-zoom key changes it: doubling.
pub fn default_zoom_step_factor() -> f32 {
    2.0
}

/// How far the step-zoom key goes before wrapping back to fitted.
pub fn default_zoom_step_max() -> f32 {
    8.0
}

/// Whether the zoom goes out past fitting the window.
pub fn default_zoom_out_past_fit() -> bool {
    false
}

/// How fast a held pan key moves the view, in screens a second.
pub fn default_pan_speed() -> f32 {
    1.5
}

/// How many photographs a screenful is.
pub fn default_page() -> usize {
    10
}

/// What is drawn under a thumbnail: the marks, which is the useful middle.
pub fn default_badges() -> String {
    "marks".to_string()
}

/// Which edge the strip of thumbnails sits against.
pub fn default_filmstrip_edge() -> String {
    "bottom".to_string()
}

/// What a new sidecar is called.
///
/// The full-name form, because it is the only one of the two that can tell a
/// raw's keywords from its JPEG twin's.
pub fn default_sidecar_naming() -> String {
    "with_extension".to_string()
}

/// `Ctrl + ,` for the settings, which is what every program on every platform
/// uses. Plain Comma is the key that walks a folded stack, so the modified one
/// is free.
pub fn default_sc_settings() -> Shortcut {
    Shortcut::new("Comma", &[MOD_CTRL])
}

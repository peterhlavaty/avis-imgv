//! The configuration file: its shape, its defaults, and how it is loaded.

pub mod bindings;
pub mod defaults;
pub mod load;
pub mod migrate;
pub mod shortcut;

use serde::{Deserialize, Serialize};

use crate::actions::Callback;

pub use defaults::*;
pub use shortcut::{build_keyboard_shortcut, Shortcut, ShortcutData};

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Config {
    /// Which build's conventions this file was written to.
    ///
    /// Absent — and so nought — in every file written before versions
    /// existed, which is exactly the set of files that needs every migration
    /// step. See [`migrate`].
    #[serde(default)]
    pub version: u32,
    pub image_view: ImageViewConfig,
    pub grid_view: GridViewConfig,
    pub general: GeneralConfig,
    pub cache: CacheConfig,
    pub slideshow: SlideshowConfig,
    pub tags: TagConfig,
    pub raw: RawConfig,
    pub cull: CullConfig,
    /// Whether something in the file on disk could not be read.
    ///
    /// A configuration that was only partly understood must never be written
    /// back: saving it would replace whatever the user had written with the
    /// defaults that stood in for it. One malformed section used to cost the
    /// whole file, and the first change made in the settings editor made that
    /// permanent.
    #[serde(skip)]
    pub partial: bool,
    /// What was brought forward from an older file, for the user to be told.
    ///
    /// A migration changes a setting somebody may have been relying on, so it
    /// says so rather than doing it quietly.
    #[serde(skip)]
    pub migrated: Vec<&'static str>,
}

impl Default for Config {
    /// A configuration built here is by definition current, so nobody
    /// starting today is told that anything moved.
    fn default() -> Self {
        Config {
            version: migrate::CURRENT,
            image_view: ImageViewConfig::default(),
            grid_view: GridViewConfig::default(),
            general: GeneralConfig::default(),
            cache: CacheConfig::default(),
            slideshow: SlideshowConfig::default(),
            tags: TagConfig::default(),
            raw: RawConfig::default(),
            cull: CullConfig::default(),
            partial: false,
            migrated: Vec::new(),
        }
    }
}

/// What to do with camera raw files.
#[derive(Deserialize, Serialize, Clone)]
pub struct RawConfig {
    /// Which half of a raw+JPEG pair is browsed, or whether to pair at all.
    ///
    /// A camera set to raw+JPEG writes two files of the same frame; browsing
    /// both means rating the shoot twice and letting the two copies disagree
    /// about what was decided.
    #[serde(default)]
    pub pair_with_jpeg: crate::organize::pairs::Prefer,
    /// Whether to show the JPEG preview the camera embedded, or develop the
    /// sensor data. Developing gives the full resolution and dynamic range and
    /// costs about a second per image.
    #[serde(default = "default_raw_source")]
    pub source: RawSource,
    /// How much work to spend demosaicing.
    #[serde(default = "default_raw_quality")]
    pub quality: RawQuality,
    /// Use the white balance the camera recorded. Without it colours come out
    /// noticeably wrong.
    #[serde(default = "default_camera_white_balance")]
    pub camera_white_balance: bool,
    /// Stretch the histogram to use the whole range.
    #[serde(default = "default_auto_brighten")]
    pub auto_brighten: bool,
    /// 0 clips blown highlights, 1 leaves them unclipped, 2 blends, and 3
    /// upwards rebuild them.
    #[serde(default = "default_highlight_mode")]
    pub highlight_mode: u8,
}

/// Which of the two pictures inside a raw file to show.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawSource {
    /// The JPEG the camera embedded: what it showed you on its own screen,
    /// and almost free to decode.
    Preview,
    /// The sensor data, developed.
    Develop,
}

/// How much work to spend demosaicing, which is most of the cost.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawQuality {
    Fast,
    Balanced,
    Best,
}

/// The star rating and tagging panel.
#[derive(Deserialize, Serialize, Clone)]
pub struct TagConfig {
    /// Tags kept permanently to hand, grouped into categories. The panel lists
    /// them in the order given here and searches both tag and category names.
    #[serde(default = "default_tag_categories")]
    pub categories: Vec<TagCategory>,
    /// How many recently used tags to remember between sessions.
    #[serde(default = "default_recent_tags")]
    pub recent_tags: usize,
    /// Starting width of the panel, in points.
    #[serde(default = "default_tag_panel_width")]
    pub panel_width: f32,

    #[serde(default = "default_sc_toggle_tag_panel")]
    pub sc_toggle_tag_panel: Shortcut,
    /// Applying a rating with a keystroke, from no stars to five.
    #[serde(default = "default_sc_rating")]
    pub sc_rating: Vec<Shortcut>,
    /// Marking a frame as kept.
    #[serde(default = "default_sc_pick")]
    pub sc_pick: Shortcut,
    /// Marking a frame as thrown out. Pressing it again puts it back.
    #[serde(default = "default_sc_reject")]
    pub sc_reject: Shortcut,
    /// Taking whichever mark a frame carries back off it.
    #[serde(default = "default_sc_unflag")]
    pub sc_unflag: Shortcut,
    /// The colour labels, in the order [`crate::metadata::xmp::Label`] lists
    /// them: red, yellow, green, blue, purple.
    #[serde(default = "default_sc_label")]
    pub sc_label: Vec<Shortcut>,
    /// Move to the next photograph after a rating, a flag or a label.
    ///
    /// Holding shift with any of those keys advances once whatever this says,
    /// so the setting decides which way round the extra keystroke goes.
    #[serde(default = "default_advance_after_marking")]
    pub advance_after_marking: bool,
    /// Turns that on and off without opening the settings.
    #[serde(default = "default_sc_toggle_advance")]
    pub sc_toggle_advance: Shortcut,
}

/// Where photographs go when they are not staying here.
#[derive(Deserialize, Serialize, Clone)]
pub struct CullConfig {
    /// Folders a photograph can be sent to with one keystroke, in the order
    /// the digit keys reach them.
    #[serde(default = "default_destinations")]
    pub destinations: Vec<Destination>,
    /// What the folder for rejected frames is called.
    ///
    /// A subfolder rather than the bin, because the bin does not reach a
    /// memory card or a network share — which is exactly where a first pass
    /// happens.
    #[serde(default = "default_rejected_folder")]
    pub rejected_folder: String,

    #[serde(default = "default_sc_move")]
    pub sc_move: Shortcut,
    #[serde(default = "default_sc_copy")]
    pub sc_copy: Shortcut,
    #[serde(default = "default_sc_reject_folder")]
    pub sc_reject_folder: Shortcut,
    #[serde(default = "default_sc_undo")]
    pub sc_undo: Shortcut,
}

impl Default for CullConfig {
    fn default() -> Self {
        CullConfig {
            destinations: default_destinations(),
            rejected_folder: default_rejected_folder(),
            sc_move: default_sc_move(),
            sc_copy: default_sc_copy(),
            sc_reject_folder: default_sc_reject_folder(),
            sc_undo: default_sc_undo(),
        }
    }
}

/// One place photographs can be sent.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Destination {
    /// What it is called on the panel.
    pub label: String,
    /// Where it is. Relative paths are taken against the open folder, so a
    /// configured `Selects` follows the shoot rather than naming one.
    pub path: String,
}

/// A named group of tags.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TagCategory {
    pub name: String,
    pub tags: Vec<String>,
}

/// How much of the machine the viewer is allowed to use to stay ahead of the
/// user.
#[derive(Deserialize, Serialize, Clone)]
pub struct CacheConfig {
    /// Ceiling on decoded pixels kept in RAM, across both views.
    #[serde(default = "default_ram_budget_mb")]
    pub ram_budget_mb: usize,
    /// Decode worker threads. Zero picks one per core, less one for the UI.
    #[serde(default = "default_decode_threads")]
    pub decode_threads: usize,
    /// How many camera thumbnails to keep on the GPU, so an image that is
    /// still being decoded has something standing in for it rather than a
    /// spinner. Zero turns that off.
    #[serde(default = "default_previews_resident")]
    pub previews_resident: usize,
    #[serde(default = "default_full_resolution_neighbours")]
    pub full_resolution_neighbours: usize,
    /// Ceiling on what the two caches may hold on the adapter, in megabytes.
    ///
    /// The counts beside it bound how *many* textures stay resident, which is
    /// not a memory bound: two hundred thumbnails and two hundred sixty
    /// megapixel photographs are the same number and a thousandfold difference
    /// in what the card is holding.
    #[serde(default = "default_gpu_budget_mb")]
    pub gpu_budget_mb: usize,
    /// How long a frame may spend moving decoded images onto the GPU.
    ///
    /// A 24 megapixel texture takes about 12ms, so this is the difference
    /// between a smooth frame rate and a stuttering one while the cache fills.
    #[serde(default = "default_upload_budget_ms")]
    pub upload_budget_ms: u64,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_output_icc_profile")]
    pub output_icc_profile: String,
    #[serde(default = "default_text_scaling")]
    pub text_scaling: f32,
    #[serde(default = "default_metadata_tags")]
    pub metadata_tags: Vec<String>,
    /// Whether to open where the last run left off.
    ///
    /// The window's size and place, the folder that was open, and — the one
    /// that earns its keep — which photograph was being looked at in each
    /// folder visited lately. A cull is rarely one sitting.
    #[serde(default = "default_restore_session")]
    pub restore_session: bool,

    #[serde(default = "default_sc_toggle_gallery")]
    pub sc_toggle_gallery: Shortcut,
    /// Cycles through the modes: image, gallery, rename, time shift.
    #[serde(default = "default_sc_next_mode")]
    pub sc_next_mode: Shortcut,
    #[serde(default = "default_sc_exit")]
    pub sc_exit: Shortcut,
    #[serde(default = "default_sc_menu")]
    pub sc_menu: Shortcut,
    #[serde(default = "default_sc_navigator")]
    pub sc_navigator: Shortcut,
    #[serde(default = "default_sc_dir_tree")]
    pub sc_dir_tree: Shortcut,
    #[serde(default = "default_sc_flatten_dir")]
    pub sc_flatten_dir: Shortcut,
    #[serde(default = "default_sc_watch_directory")]
    pub sc_watch_directory: Shortcut,
    #[serde(default = "default_sc_toggle_side_panel")]
    pub sc_toggle_side_panel: Shortcut,
    /// Sends the picture on screen to the platform's bin.
    #[serde(default = "default_sc_delete")]
    pub sc_delete: Shortcut,
    /// Deletes it outright, for the cards and shares that have no bin.
    #[serde(default = "default_sc_delete_permanently")]
    pub sc_delete_permanently: Shortcut,
    /// Fills the screen and gives it back.
    #[serde(default = "default_sc_fullscreen")]
    pub sc_fullscreen: Shortcut,
    /// Shows and hides the bar that narrows and orders the folder.
    #[serde(default = "default_sc_filter")]
    pub sc_filter: Shortcut,
    /// Sets the rules aside without forgetting them, so "what did I hide?" is
    /// one key and answering it costs nothing.
    #[serde(default = "default_sc_suspend_filter")]
    pub sc_suspend_filter: Shortcut,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ImageViewConfig {
    /// Where on the photograph its own details are drawn, if anywhere.
    ///
    /// The status bar says the same things and is in the wrong place for them
    /// when the viewer is fullscreen for a slideshow or a review: there is no
    /// chrome then, and the eye is on the picture.
    #[serde(default)]
    pub overlay_corner: crate::view::image_view::overlay::Corner,
    /// What it says, in the template grammar, one line per line.
    #[serde(default = "default_overlay_format")]
    pub overlay_format: String,
    #[serde(default = "default_overlay_text_size")]
    pub overlay_text_size: f32,
    /// Moves what the photograph says about itself round the corners, and off.
    #[serde(default = "default_sc_overlay")]
    pub sc_overlay: Shortcut,
    /// Images decoded either side of the one on screen.
    #[serde(default = "default_nr_loaded_images")]
    pub nr_loaded_images: usize,
    /// Images kept as GPU textures, ready to draw without any upload.
    #[serde(default = "default_gpu_resident_images")]
    pub gpu_resident_images: usize,
    /// Cap on the longest edge of a decoded image. Zero means the largest the
    /// GPU accepts.
    #[serde(default = "default_max_image_edge")]
    pub max_image_edge: u32,
    #[serde(default = "default_nr_images_shown")]
    pub nr_images_shown: usize,
    #[serde(default = "default_should_wait")]
    pub should_wait: bool,
    #[serde(default = "default_frame_size_relative_to_image")]
    pub frame_size_relative_to_image: f32,
    #[serde(default = "default_scroll_navigation")]
    pub scroll_navigation: bool,
    /// Whether a photograph smaller than the window is enlarged to fill it.
    ///
    /// The one that needs it is a raw file's embedded copy: a DNG written by
    /// Camera Raw carries a 256 pixel preview and nothing else, and drawn at
    /// its own size it is a postage stamp in the middle of the screen.
    #[serde(default = "default_enlarge_to_fit")]
    pub enlarge_to_fit: bool,
    #[serde(default = "default_name_format")]
    pub name_format: String,
    #[serde(default = "default_user_actions")]
    pub user_actions: Vec<UserAction>,
    #[serde(default = "default_ctx_menu")]
    pub context_menu: Vec<ContextMenuEntry>,

    #[serde(default = "default_sc_fit")]
    pub sc_fit: Shortcut,
    #[serde(default = "default_sc_frame")]
    pub sc_frame: Shortcut,
    #[serde(default = "default_sc_zoom")]
    pub sc_zoom: Shortcut,
    #[serde(default = "default_sc_next")]
    pub sc_next: Shortcut,
    #[serde(default = "default_sc_prev")]
    pub sc_prev: Shortcut,
    #[serde(default = "default_sc_one_to_one")]
    pub sc_one_to_one: Shortcut,
    /// Puts this image at the zoom and position the last one was left at.
    #[serde(default = "default_sc_repeat_place")]
    pub sc_repeat_place: Shortcut,
    #[serde(default = "default_sc_fit_horizontal")]
    pub sc_fit_horizontal: Shortcut,
    #[serde(default = "default_sc_fit_vertical")]
    pub sc_fit_vertical: Shortcut,
    #[serde(default = "default_sc_fit_maximize")]
    pub sc_fit_maximize: Shortcut,
    #[serde(default = "default_sc_latch_fit_maximize")]
    pub sc_latch_fit_maximize: Shortcut,
    #[serde(default = "default_sc_more_images_shown")]
    pub sc_more_images_shown: Shortcut,
    #[serde(default = "default_sc_less_images_shown")]
    pub sc_less_images_shown: Shortcut,
    /// Pins the photograph on screen and its neighbours side by side, sharing
    /// one zoom and one pan, until one of them wins.
    #[serde(default = "default_sc_compare")]
    pub sc_compare: Shortcut,
    #[serde(default = "default_sc_zoom_in")]
    pub sc_zoom_in: Shortcut,
    #[serde(default = "default_sc_zoom_out")]
    pub sc_zoom_out: Shortcut,
    /// Held rather than tapped: panning follows the key for as long as it is
    /// down, which is why these are read separately from the shortcuts.
    #[serde(default = "default_sc_pan_up")]
    pub sc_pan_up: Shortcut,
    #[serde(default = "default_sc_pan_down")]
    pub sc_pan_down: Shortcut,
    #[serde(default = "default_sc_pan_left")]
    pub sc_pan_left: Shortcut,
    #[serde(default = "default_sc_pan_right")]
    pub sc_pan_right: Shortcut,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GridViewConfig {
    #[serde(default = "default_images_per_row")]
    pub images_per_row: usize,
    /// How wide a cell's picture is against its height.
    ///
    /// Cells used to be square, which for a folder of landscape photographs
    /// left about forty-four per cent of the contact sheet drawn in grey.
    /// 1.5 is the three-to-two most cameras shoot; 1.0 brings the squares
    /// back.
    #[serde(default = "default_cell_aspect")]
    pub cell_aspect: f32,
    #[serde(default = "default_preloaded_rows")]
    pub preloaded_rows: usize,
    /// Longest edge of a decoded thumbnail.
    #[serde(default = "default_thumbnail_resolution")]
    pub thumbnail_resolution: u32,
    /// Thumbnails kept as GPU textures.
    #[serde(default = "default_gpu_resident_thumbnails")]
    pub gpu_resident_thumbnails: usize,
    #[serde(default = "default_ctx_menu")]
    pub context_menu: Vec<ContextMenuEntry>,

    #[serde(default = "default_sc_scroll")]
    pub sc_scroll: Shortcut,
    #[serde(default = "default_sc_more_per_row")]
    pub sc_more_per_row: Shortcut,
    #[serde(default = "default_sc_less_per_row")]
    pub sc_less_per_row: Shortcut,
    /// Cycles what is drawn under each thumbnail: nothing, the marks, or the
    /// marks and the file name.
    #[serde(default = "default_sc_cycle_badges")]
    pub sc_cycle_badges: Shortcut,
    /// What the line under each thumbnail says, in the template grammar.
    ///
    /// The file name by default, which is what it always said; anything the
    /// grammar reaches works, so a sheet can be labelled by shutter speed
    /// while somebody is looking for the one that was not blurred.
    #[serde(default = "default_caption_format")]
    pub caption_format: String,
    #[serde(default = "default_sc_select")]
    pub sc_select: Shortcut,
    #[serde(default = "default_sc_select_all")]
    pub sc_select_all: Shortcut,
}

/// What the slideshow does with a picture while it is up.
#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Motion {
    /// Show the whole picture and leave it alone.
    Still,
    /// Fill the screen and drift slowly further in.
    #[default]
    Zoom,
    /// Fill the screen and travel across, so the whole picture has been seen
    /// by the time the next one comes up.
    Reveal,
}

impl Motion {
    pub const ALL: &'static [Motion] = &[Motion::Still, Motion::Zoom, Motion::Reveal];

    pub fn label(self) -> &'static str {
        match self {
            Motion::Still => "Hold still",
            Motion::Zoom => "Drift inwards",
            Motion::Reveal => "Travel across",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Motion::Still => "The whole picture, fitted to the screen, not moving.",
            Motion::Zoom => "Fills the screen and creeps closer while it is up.",
            Motion::Reveal => {
                "Fills the screen at its own shape and travels across it, so the whole \
                 picture has been seen by the time the next one comes up."
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SlideshowConfig {
    #[serde(default = "default_seconds_per_image")]
    pub seconds_per_image: u64,
    #[serde(default = "default_percent_zoom")]
    pub percent_zoom: f32,
    #[serde(default)]
    pub motion: Motion,
    #[serde(default = "default_start_with_frame_enabled")]
    pub start_with_frame_enabled: bool,
    #[serde(default = "default_image_frame_background_color_override")]
    pub image_frame_background_color_override: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UserAction {
    pub shortcut: Shortcut,
    pub exec: String,
    pub callback: Option<Callback>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ContextMenuEntry {
    pub description: String,
    pub exec: String,
    pub callback: Option<Callback>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            output_icc_profile: default_output_icc_profile(),
            text_scaling: default_text_scaling(),
            metadata_tags: default_metadata_tags(),
            restore_session: default_restore_session(),
            sc_toggle_gallery: default_sc_toggle_gallery(),
            sc_next_mode: default_sc_next_mode(),
            sc_toggle_side_panel: default_sc_toggle_side_panel(),
            sc_delete: default_sc_delete(),
            sc_delete_permanently: default_sc_delete_permanently(),
            sc_fullscreen: default_sc_fullscreen(),
            sc_filter: default_sc_filter(),
            sc_suspend_filter: default_sc_suspend_filter(),
            sc_exit: default_sc_exit(),
            sc_menu: default_sc_menu(),
            sc_navigator: default_sc_navigator(),
            sc_dir_tree: default_sc_dir_tree(),
            sc_flatten_dir: default_sc_flatten_dir(),
            sc_watch_directory: default_sc_watch_directory(),
        }
    }
}

impl Default for ImageViewConfig {
    fn default() -> Self {
        ImageViewConfig {
            nr_loaded_images: default_nr_loaded_images(),
            gpu_resident_images: default_gpu_resident_images(),
            max_image_edge: default_max_image_edge(),
            nr_images_shown: default_nr_images_shown(),
            should_wait: default_should_wait(),
            frame_size_relative_to_image: default_frame_size_relative_to_image(),
            scroll_navigation: default_scroll_navigation(),
            enlarge_to_fit: default_enlarge_to_fit(),
            user_actions: default_user_actions(),
            context_menu: default_ctx_menu(),
            name_format: default_name_format(),
            overlay_corner: crate::view::image_view::overlay::Corner::default(),
            overlay_format: default_overlay_format(),
            overlay_text_size: default_overlay_text_size(),
            sc_overlay: default_sc_overlay(),

            sc_fit: default_sc_fit(),
            sc_frame: default_sc_frame(),
            sc_zoom: default_sc_zoom(),
            sc_next: default_sc_next(),
            sc_prev: default_sc_prev(),
            sc_one_to_one: default_sc_one_to_one(),
            sc_repeat_place: default_sc_repeat_place(),
            sc_fit_vertical: default_sc_fit_vertical(),
            sc_fit_horizontal: default_sc_fit_horizontal(),
            sc_fit_maximize: default_sc_fit_maximize(),
            sc_latch_fit_maximize: default_sc_latch_fit_maximize(),
            sc_more_images_shown: default_sc_more_images_shown(),
            sc_less_images_shown: default_sc_less_images_shown(),
            sc_compare: default_sc_compare(),
            sc_zoom_in: default_sc_zoom_in(),
            sc_zoom_out: default_sc_zoom_out(),
            sc_pan_up: default_sc_pan_up(),
            sc_pan_down: default_sc_pan_down(),
            sc_pan_left: default_sc_pan_left(),
            sc_pan_right: default_sc_pan_right(),
        }
    }
}

impl Default for GridViewConfig {
    fn default() -> Self {
        GridViewConfig {
            images_per_row: default_images_per_row(),
            cell_aspect: default_cell_aspect(),
            preloaded_rows: default_preloaded_rows(),
            thumbnail_resolution: default_thumbnail_resolution(),
            gpu_resident_thumbnails: default_gpu_resident_thumbnails(),
            context_menu: default_ctx_menu(),

            sc_scroll: default_sc_scroll(),
            sc_more_per_row: default_sc_more_per_row(),
            sc_less_per_row: default_sc_less_per_row(),
            sc_cycle_badges: default_sc_cycle_badges(),
            caption_format: default_caption_format(),
            sc_select: default_sc_select(),
            sc_select_all: default_sc_select_all(),
        }
    }
}

impl Default for RawConfig {
    fn default() -> Self {
        RawConfig {
            pair_with_jpeg: crate::organize::pairs::Prefer::default(),
            source: default_raw_source(),
            quality: default_raw_quality(),
            camera_white_balance: default_camera_white_balance(),
            auto_brighten: default_auto_brighten(),
            highlight_mode: default_highlight_mode(),
        }
    }
}

impl Default for TagConfig {
    fn default() -> Self {
        TagConfig {
            categories: default_tag_categories(),
            recent_tags: default_recent_tags(),
            panel_width: default_tag_panel_width(),
            sc_toggle_tag_panel: default_sc_toggle_tag_panel(),
            sc_rating: default_sc_rating(),
            sc_pick: default_sc_pick(),
            sc_reject: default_sc_reject(),
            sc_unflag: default_sc_unflag(),
            sc_label: default_sc_label(),
            advance_after_marking: default_advance_after_marking(),
            sc_toggle_advance: default_sc_toggle_advance(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            ram_budget_mb: default_ram_budget_mb(),
            decode_threads: default_decode_threads(),
            previews_resident: default_previews_resident(),
            full_resolution_neighbours: default_full_resolution_neighbours(),
            gpu_budget_mb: default_gpu_budget_mb(),
            upload_budget_ms: default_upload_budget_ms(),
        }
    }
}

impl Default for SlideshowConfig {
    fn default() -> Self {
        SlideshowConfig {
            seconds_per_image: default_seconds_per_image(),
            motion: Motion::default(),
            percent_zoom: default_percent_zoom(),
            start_with_frame_enabled: default_start_with_frame_enabled(),
            image_frame_background_color_override: default_image_frame_background_color_override(),
        }
    }
}

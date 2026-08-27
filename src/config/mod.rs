//! The configuration file: its shape, its defaults, and how it is loaded.

pub mod defaults;
pub mod load;
pub mod shortcut;

use serde::{Deserialize, Serialize};

use crate::actions::Callback;

pub use defaults::*;
pub use shortcut::{build_keyboard_shortcut, Shortcut, ShortcutData};

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub image_view: ImageViewConfig,
    pub grid_view: GridViewConfig,
    pub general: GeneralConfig,
    pub cache: CacheConfig,
    pub slideshow: SlideshowConfig,
    pub tags: TagConfig,
    pub raw: RawConfig,
}

/// What to do with camera raw files.
#[derive(Deserialize, Serialize, Clone)]
pub struct RawConfig {
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
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ImageViewConfig {
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
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SlideshowConfig {
    #[serde(default = "default_seconds_per_image")]
    pub seconds_per_image: u64,
    #[serde(default = "default_percent_zoom")]
    pub percent_zoom: f32,
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
            sc_toggle_gallery: default_sc_toggle_gallery(),
            sc_next_mode: default_sc_next_mode(),
            sc_toggle_side_panel: default_sc_toggle_side_panel(),
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
            user_actions: default_user_actions(),
            context_menu: default_ctx_menu(),
            name_format: default_name_format(),

            sc_fit: default_sc_fit(),
            sc_frame: default_sc_frame(),
            sc_zoom: default_sc_zoom(),
            sc_next: default_sc_next(),
            sc_prev: default_sc_prev(),
            sc_one_to_one: default_sc_one_to_one(),
            sc_fit_vertical: default_sc_fit_vertical(),
            sc_fit_horizontal: default_sc_fit_horizontal(),
            sc_fit_maximize: default_sc_fit_maximize(),
            sc_latch_fit_maximize: default_sc_latch_fit_maximize(),
            sc_more_images_shown: default_sc_more_images_shown(),
            sc_less_images_shown: default_sc_less_images_shown(),
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
            preloaded_rows: default_preloaded_rows(),
            thumbnail_resolution: default_thumbnail_resolution(),
            gpu_resident_thumbnails: default_gpu_resident_thumbnails(),
            context_menu: default_ctx_menu(),

            sc_scroll: default_sc_scroll(),
            sc_more_per_row: default_sc_more_per_row(),
            sc_less_per_row: default_sc_less_per_row(),
        }
    }
}

impl Default for RawConfig {
    fn default() -> Self {
        RawConfig {
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
            upload_budget_ms: default_upload_budget_ms(),
        }
    }
}

impl Default for SlideshowConfig {
    fn default() -> Self {
        SlideshowConfig {
            seconds_per_image: default_seconds_per_image(),
            percent_zoom: default_percent_zoom(),
            start_with_frame_enabled: default_start_with_frame_enabled(),
            image_frame_background_color_override: default_image_frame_background_color_override(),
        }
    }
}

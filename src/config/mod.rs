//! The configuration file: its shape, its defaults, and how it is loaded.

pub mod bindings;
pub mod browsing;
pub mod defaults;
pub mod history;
pub mod load;
pub mod migrate;
pub mod mouse;
pub mod registry;
pub mod shortcut;

use serde::{Deserialize, Serialize};

use crate::actions::Callback;

pub use browsing::{BrowsingConfig, Confirmations, GroupConfig, MenuConfig, PanelsAtStart};
pub use defaults::*;
pub use history::{HistoryConfig, Undoes};
pub use mouse::{DragButton, MouseConfig, WheelJob};
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
    /// What a folder opens as: its order, its filter, whether it is stacked.
    #[serde(default)]
    pub browsing: BrowsingConfig,
    /// What counts as one run of frames, read by both surfaces that detect one.
    #[serde(default)]
    pub group: GroupConfig,
    /// What the second button offers.
    #[serde(default)]
    pub menus: MenuConfig,
    /// What the pointer does.
    #[serde(default)]
    pub mouse: MouseConfig,
    /// What was done this run, and the keys that walk back through it.
    #[serde(default)]
    pub history: HistoryConfig,
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
    /// The document this configuration was read from, keys and all.
    ///
    /// Kept so that a save can be a merge rather than a replacement. A key
    /// this build has never heard of belongs to whoever wrote it, and dropping
    /// it on the way out is how one build's settings are lost to another —
    /// Geeqie's defect. `None` when nothing was read from a file.
    #[serde(skip)]
    pub document: Option<serde_json::Map<String, serde_json::Value>>,
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
            browsing: BrowsingConfig::default(),
            group: GroupConfig::default(),
            menus: MenuConfig::default(),
            mouse: MouseConfig::default(),
            history: HistoryConfig::default(),
            partial: false,
            migrated: Vec::new(),
            document: None,
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

impl RawSource {
    pub const ALL: &'static [RawSource] = &[RawSource::Preview, RawSource::Develop];

    /// The word the file holds, which is what the registry is keyed on and what
    /// a forum answer quotes.
    pub fn value(self) -> &'static str {
        match self {
            RawSource::Preview => "preview",
            RawSource::Develop => "develop",
        }
    }

    pub fn of(value: &str) -> Option<RawSource> {
        RawSource::ALL
            .iter()
            .copied()
            .find(|it| it.value() == value)
    }
}

impl RawQuality {
    pub const ALL: &'static [RawQuality] =
        &[RawQuality::Fast, RawQuality::Balanced, RawQuality::Best];

    pub fn value(self) -> &'static str {
        match self {
            RawQuality::Fast => "fast",
            RawQuality::Balanced => "balanced",
            RawQuality::Best => "best",
        }
    }

    pub fn of(value: &str) -> Option<RawQuality> {
        RawQuality::ALL
            .iter()
            .copied()
            .find(|it| it.value() == value)
    }
}

/// The star rating and tagging panel.
#[derive(Deserialize, Serialize, Clone)]
pub struct TagConfig {
    /// Tags kept permanently to hand, grouped into categories. The panel lists
    /// them in the order given here and searches both tag and category names.
    #[serde(default = "default_tag_categories")]
    pub categories: Vec<TagCategory>,
    /// A keyword list exported from another program, read at startup and added
    /// to the categories above.
    ///
    /// Every photo application can export its keywords as an indented text
    /// file, and a photographer with years of them in Lightroom or digiKam
    /// should not have to type them again here. Indentation makes the
    /// hierarchy, and a keyword filed under levels is written to the sidecar
    /// with them.
    #[serde(default)]
    pub catalog_file: Option<String>,
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
    /// What a sidecar this viewer creates is called.
    ///
    /// Both forms are *read*, most specific first, and a sidecar that already
    /// exists is edited rather than joined by a second. This is only what gets
    /// created for a photograph that has none. `photo.cr2.xmp` is the default
    /// because it is the only one of the two that can tell a raw's keywords
    /// from its JPEG twin's, which is a correctness property rather than a
    /// preference.
    #[serde(default = "default_sidecar_naming")]
    pub sidecar_naming: String,
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
    /// Which of the reversible things ask first.
    #[serde(default)]
    pub confirm: Confirmations,
    /// Which bin the delete key means: `system` or `folder`.
    ///
    /// The platform's stays the default. It is what Delete means everywhere
    /// else, and a viewer that quietly means something else by it is a viewer
    /// somebody has to be told about.
    #[serde(default = "default_bin")]
    pub bin: String,
    /// Where the viewer's own bin is, when that is the one in use.
    ///
    /// Absolute, or nothing for the folder under the local data directory. One
    /// bin rather than one per shoot: a bin relative to the open folder would
    /// be a different bin in every folder, and the question asked at closing
    /// time would be about whichever one happened to be open.
    #[serde(default = "default_bin_folder")]
    pub bin_folder: Option<String>,
    /// Whether a bin left with something in it is asked about on the way out.
    #[serde(default = "default_ask_to_empty_the_bin")]
    pub ask_to_empty_the_bin: bool,

    #[serde(default = "default_sc_move")]
    pub sc_move: Shortcut,
    #[serde(default = "default_sc_copy")]
    pub sc_copy: Shortcut,
    #[serde(default = "default_sc_reject_folder")]
    pub sc_reject_folder: Shortcut,
    #[serde(default = "default_sc_put_back")]
    pub sc_put_back: Shortcut,
}

impl Default for CullConfig {
    fn default() -> Self {
        CullConfig {
            destinations: default_destinations(),
            rejected_folder: default_rejected_folder(),
            confirm: Confirmations::default(),
            bin: default_bin(),
            bin_folder: default_bin_folder(),
            ask_to_empty_the_bin: default_ask_to_empty_the_bin(),
            sc_move: default_sc_move(),
            sc_copy: default_sc_copy(),
            sc_reject_folder: default_sc_reject_folder(),
            sc_put_back: default_sc_put_back(),
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
    /// Which mode a launch starts in.
    ///
    /// The precedence, which the row on the page states: a path on the command
    /// line, then the restored session, then the startup folder, then the
    /// working directory.
    #[serde(default = "default_start_in")]
    pub start_in: String,
    /// Whether the window fills the screen when it opens.
    ///
    /// `--fullscreen` says the same thing for one launch; this says it for
    /// every launch.
    #[serde(default)]
    pub start_fullscreen: bool,
    /// The folder to open when nothing else names one.
    ///
    /// Reached only when no path was given and there is no session to restore,
    /// which is what stops the viewer opening the working directory of whatever
    /// launched it — nobody's choice.
    #[serde(default)]
    pub start_folder: Option<String>,
    /// Which panels a launch starts with.
    #[serde(default)]
    pub panels_at_start: PanelsAtStart,
    /// Starting width of the metadata panel, in points.
    #[serde(default = "default_side_panel_width")]
    pub side_panel_width: f32,
    /// Light or dark.
    ///
    /// The theme was hardcoded to dark above a reason that is sound and too
    /// wide: what surrounds the *photograph* is the backdrop, which is a
    /// separate value the theme does not touch.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// The grey behind the photograph, as a hex colour.
    ///
    /// Separate from the theme, and the thing the theme's own argument is
    /// really about: a light interface does not have to mean a light ground
    /// under the picture.
    #[serde(default = "default_backdrop")]
    pub backdrop: String,
    /// Which page the settings window was last left on.
    ///
    /// Kept here rather than in the session file, because `on_exit` returns
    /// early when `restore_session` is off — a preference kept there is a
    /// preference some people silently do not have. A key the window writes and
    /// nobody sets, like `version`.
    #[serde(default)]
    pub last_settings_page: String,

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
    /// Shows or hides the strip of thumbnails under the photograph.
    #[serde(default = "default_sc_filmstrip")]
    pub sc_filmstrip: Shortcut,
    /// Shows the folder stacked — one cell per run of frames — or puts every
    /// frame back.
    #[serde(default = "default_sc_stacks")]
    pub sc_stacks: Shortcut,
    /// Turn the photograph a quarter, anticlockwise and clockwise.
    ///
    /// The turn is written to the sidecar; the photograph itself is never
    /// touched.
    #[serde(default = "default_sc_turn_left")]
    pub sc_turn_left: Shortcut,
    #[serde(default = "default_sc_turn_right")]
    pub sc_turn_right: Shortcut,
    /// Opens or closes the stack the cursor is in.
    #[serde(default = "default_sc_toggle_stack")]
    pub sc_toggle_stack: Shortcut,
    /// Changes which frame stands for the stack the cursor is in.
    #[serde(default = "default_sc_standing_back")]
    pub sc_standing_back: Shortcut,
    #[serde(default = "default_sc_standing_forward")]
    pub sc_standing_forward: Shortcut,
    /// Steps from one run of frames to the next, over a burst rather than
    /// through it.
    #[serde(default = "default_sc_previous_stack")]
    pub sc_previous_stack: Shortcut,
    #[serde(default = "default_sc_next_stack")]
    pub sc_next_stack: Shortcut,
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
    /// Opens the settings window on the page it was last left on.
    ///
    /// Plain Comma is the key that walks the frames of a folded stack, so the
    /// modified one is free — and like every other key it is a registry row
    /// rather than a hardcoded one.
    #[serde(default = "default_sc_settings")]
    pub sc_settings: Shortcut,
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
    /// Marks what has clipped, then what is in focus, then nothing.
    #[serde(default = "default_sc_marks")]
    pub sc_marks: Shortcut,
    /// Magnifies until the marked area fills the panel.
    #[serde(default = "default_sc_zoom_to_area")]
    pub sc_zoom_to_area: Shortcut,
    /// How far the rest of the photograph is darkened while an area is marked,
    /// out of a hundred.
    #[serde(default = "default_marked_area_dim")]
    pub marked_area_dim: u8,
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
    /// Whether a photograph smaller than the window is enlarged to fill it.
    ///
    /// The one that needs it is a raw file's embedded copy: a DNG written by
    /// Camera Raw carries a 256 pixel preview and nothing else, and drawn at
    /// its own size it is a postage stamp in the middle of the screen.
    #[serde(default = "default_enlarge_to_fit")]
    pub enlarge_to_fit: bool,
    /// What a photograph is drawn at on the frame it first appears.
    ///
    /// Fitted, by default, which is what a viewer has always done. The other
    /// two are for the two ways a shoot is actually gone through: filling the
    /// window judges a composition on the whole screen, and its own size is
    /// the magnification focus is judged at.
    #[serde(default)]
    pub opening: crate::view::image_view::opening::Opening,
    /// Whether the magnification carries from one photograph to the next.
    ///
    /// Off, and reached from the status bar rather than from here: it is a way
    /// of working for the next ten minutes — a burst gone through at a hundred
    /// per cent — rather than a preference. It overrides
    /// [`Self::opening`] while it is on, and where each photograph was left
    /// with it.
    #[serde(default)]
    pub keep_zoom: bool,
    /// Whether where in the photograph the view is carries to the next one.
    ///
    /// The other half of [`Self::keep_zoom`], and separate from it because
    /// the same eye in every frame of a burst and the same magnification are
    /// two different asks: a hand-held sequence moves, and following it is
    /// what panning is for.
    #[serde(default)]
    pub keep_pan: bool,
    /// Whether the zoom goes out past fitting the window.
    ///
    /// It does not, by default, and stops exactly at the fit. Below it the
    /// photograph has a border on all four sides and there is nothing more to
    /// see, so every notch spent getting there is a notch spent getting back:
    /// the sibling of [`Self::enlarge_to_fit`], which is the same argument
    /// about a photograph too small to fill the window in the first place.
    #[serde(default = "default_zoom_out_past_fit")]
    pub zoom_out_past_fit: bool,
    /// How much one press of the zoom keys changes the magnification.
    #[serde(default = "default_zoom_step")]
    pub zoom_step: f32,
    /// How much one press of the step-zoom key changes it.
    ///
    /// Doubling, by default: that key exists to get from fitted to something
    /// worth judging in as few presses as possible.
    #[serde(default = "default_zoom_step_factor")]
    pub zoom_step_factor: f32,
    /// How far the step-zoom key goes before wrapping back to fitted.
    #[serde(default = "default_zoom_step_max")]
    pub zoom_step_max: f32,
    /// How fast a held pan key moves the view, in screens a second.
    #[serde(default = "default_pan_speed")]
    pub pan_speed: f32,
    /// How many photographs a screenful is, for the keys that walk a long
    /// folder quickly.
    #[serde(default = "default_page")]
    pub page: usize,
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
    #[serde(default = "default_sc_cycle_opening")]
    pub sc_cycle_opening: Shortcut,
    #[serde(default = "default_sc_keep_zoom")]
    pub sc_keep_zoom: Shortcut,
    #[serde(default = "default_sc_keep_pan")]
    pub sc_keep_pan: Shortcut,
    #[serde(default = "default_sc_more_images_shown")]
    pub sc_more_images_shown: Shortcut,
    #[serde(default = "default_sc_less_images_shown")]
    pub sc_less_images_shown: Shortcut,
    /// Pins the photograph on screen and its neighbours side by side, sharing
    /// one zoom and one pan, until one of them wins.
    #[serde(default = "default_sc_compare")]
    pub sc_compare: Shortcut,
    /// Takes the focused photograph out of a comparison.
    ///
    /// A binding rather than a bare key, because it was read as `/` with no
    /// modifiers: on the Slovak, German and French layouts that is Shift and a
    /// digit, so the key could neither be pressed nor changed.
    #[serde(default = "default_sc_drop_pane")]
    pub sc_drop_pane: Shortcut,
    /// Puts the cursor in the "go to" box in the status bar.
    ///
    /// The box surrenders focus if it gains it without a click, because Tab
    /// means "the other pane" while comparing and this is the first widget in
    /// the window. Sound reasoning, and it left a control that could not be
    /// operated without a mouse.
    #[serde(default = "default_sc_go_to")]
    pub sc_go_to: Shortcut,
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
    /// How tall the strip of thumbnails under the image view is, in points.
    ///
    /// Zero turns it off, which is the default: it is a second row of pixels
    /// competing with the photograph for the window, and somebody who wants it
    /// wants it deliberately.
    #[serde(default = "default_filmstrip_height")]
    pub filmstrip_height: f32,
    /// What the line under each thumbnail says, in the template grammar.
    ///
    /// The file name by default, which is what it always said; anything the
    /// grammar reaches works, so a sheet can be labelled by shutter speed
    /// while somebody is looking for the one that was not blurred.
    #[serde(default = "default_caption_format")]
    pub caption_format: String,
    /// What is drawn under each thumbnail when a folder opens. The key cycles
    /// it for the session.
    #[serde(default = "default_badges")]
    pub badges: String,
    /// Whether the strip of thumbnails is up when a folder opens.
    ///
    /// Split out of `filmstrip_height`, which stored a height and a visibility
    /// in one number — which is why the key that shows the strip did nothing on
    /// a fresh install: the default height is zero.
    #[serde(default)]
    pub filmstrip_visible: bool,
    /// Which edge of the window the strip sits against.
    #[serde(default = "default_filmstrip_edge")]
    pub filmstrip_edge: String,
    /// Whether a single click on a cell opens that photograph.
    ///
    /// Off: a click picks out and a double click opens. A culling tool's
    /// contact sheet is a surface you act *on*, and a plain click that closes
    /// it contradicts the cursor, the selection, Ctrl-click, Shift-click, Space
    /// and Enter all at once.
    #[serde(default)]
    pub click_opens: bool,
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

    /// The word the file holds.
    pub fn value(self) -> &'static str {
        match self {
            Motion::Still => "still",
            Motion::Zoom => "zoom",
            Motion::Reveal => "reveal",
        }
    }

    pub fn of(value: &str) -> Option<Motion> {
        Motion::ALL.iter().copied().find(|it| it.value() == value)
    }

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
            start_in: default_start_in(),
            start_fullscreen: false,
            start_folder: None,
            panels_at_start: PanelsAtStart::default(),
            side_panel_width: default_side_panel_width(),
            theme: default_theme(),
            backdrop: default_backdrop(),
            last_settings_page: String::new(),
            sc_toggle_gallery: default_sc_toggle_gallery(),
            sc_next_mode: default_sc_next_mode(),
            sc_toggle_side_panel: default_sc_toggle_side_panel(),
            sc_filmstrip: default_sc_filmstrip(),
            sc_stacks: default_sc_stacks(),
            sc_turn_left: default_sc_turn_left(),
            sc_turn_right: default_sc_turn_right(),
            sc_toggle_stack: default_sc_toggle_stack(),
            sc_standing_back: default_sc_standing_back(),
            sc_standing_forward: default_sc_standing_forward(),
            sc_previous_stack: default_sc_previous_stack(),
            sc_next_stack: default_sc_next_stack(),
            sc_delete: default_sc_delete(),
            sc_delete_permanently: default_sc_delete_permanently(),
            sc_fullscreen: default_sc_fullscreen(),
            sc_filter: default_sc_filter(),
            sc_settings: default_sc_settings(),
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
            enlarge_to_fit: default_enlarge_to_fit(),
            opening: crate::view::image_view::opening::Opening::default(),
            keep_zoom: false,
            keep_pan: false,
            zoom_out_past_fit: default_zoom_out_past_fit(),
            zoom_step: default_zoom_step(),
            zoom_step_factor: default_zoom_step_factor(),
            zoom_step_max: default_zoom_step_max(),
            pan_speed: default_pan_speed(),
            page: default_page(),
            user_actions: default_user_actions(),
            context_menu: default_ctx_menu(),
            name_format: default_name_format(),
            overlay_corner: crate::view::image_view::overlay::Corner::default(),
            overlay_format: default_overlay_format(),
            overlay_text_size: default_overlay_text_size(),
            sc_overlay: default_sc_overlay(),
            sc_marks: default_sc_marks(),
            sc_zoom_to_area: default_sc_zoom_to_area(),
            marked_area_dim: default_marked_area_dim(),

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
            sc_cycle_opening: default_sc_cycle_opening(),
            sc_keep_zoom: default_sc_keep_zoom(),
            sc_keep_pan: default_sc_keep_pan(),
            sc_more_images_shown: default_sc_more_images_shown(),
            sc_less_images_shown: default_sc_less_images_shown(),
            sc_compare: default_sc_compare(),
            sc_drop_pane: default_sc_drop_pane(),
            sc_go_to: default_sc_go_to(),
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
            badges: default_badges(),
            filmstrip_visible: false,
            filmstrip_edge: default_filmstrip_edge(),
            click_opens: false,
            filmstrip_height: default_filmstrip_height(),
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
            catalog_file: None,
            recent_tags: default_recent_tags(),
            panel_width: default_tag_panel_width(),
            sc_toggle_tag_panel: default_sc_toggle_tag_panel(),
            sc_rating: default_sc_rating(),
            sc_pick: default_sc_pick(),
            sc_reject: default_sc_reject(),
            sc_unflag: default_sc_unflag(),
            sc_label: default_sc_label(),
            advance_after_marking: default_advance_after_marking(),
            sidecar_naming: default_sidecar_naming(),
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

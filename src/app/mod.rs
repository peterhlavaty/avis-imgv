//! The application: which folder is open, which view shows it, and the wiring
//! between them.

mod about;
pub mod benchmark;
mod chrome;
mod conflict;
mod cull;
pub mod input;
pub mod mode;
pub mod panels;
mod settings;
pub mod stacking;
mod stores;
pub mod tagging;
mod verbs;
mod views;
pub mod watcher;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, ViewportCommand};

use crate::actions::Callback;
use crate::annotations::{AnnotationStore, Catalog, RecentTags};
use crate::app::stacking::Stacking;
use crate::cache::loader::Loader;
use crate::config::{Config, GeneralConfig, TagConfig};
use crate::crawler;
use crate::organize::journal::Journal;
use crate::organize::pairs::Pairs;
use crate::session::{Geometry, Session};
use crate::ui::destinations::{Asking, Errand, Slot};
use crate::ui::tag_panel;
use crate::ui::{cheat_sheet, filter_bar, keys, notice::Notices, perf_metrics::PerfMetrics, theme};
use crate::view::image_view::bottom_bar::Marks;
use crate::view::narrow::Narrowing;
use crate::view::organize::OrganizeView;
use crate::view::{GridView, ImageView};

use benchmark::Benchmark;
use input::{Command, Overlay};
use mode::Mode;

/// Images a benchmark run walks through before reporting.
const BENCHMARK_IMAGES: usize = 500;

pub struct App {
    image_view: ImageView,
    grid_view: GridView,
    /// The folder jobs: renaming, and correcting a camera clock. Built lazily,
    /// because most sessions never open one and starting it reads the folder.
    organize_view: OrganizeView,
    mode: Mode,
    /// The images currently open, before either view partitions them.
    paths: Vec<PathBuf>,
    base_path: PathBuf,
    /// Whether sub-directories are folded into the open collection.
    flattened: bool,
    menu_visible: bool,
    side_panel_visible: bool,
    metrics_visible: bool,
    overlay: Option<Overlay>,
    navigator_path: String,
    watcher: watcher::DirectoryWatcher,
    perf_metrics: PerfMetrics,
    config: GeneralConfig,
    /// Star ratings and tags, kept beside the images in XMP sidecars.
    annotations: AnnotationStore,
    catalog: Catalog,
    recent_tags: RecentTags,
    tag_panel: tag_panel::State,
    tag_panel_visible: bool,
    tag_config: TagConfig,
    /// Set by `--benchmark`: walk the folder as fast as it will go, report,
    /// and quit.
    benchmark: Option<Benchmark>,
    /// The whole configuration, kept so the keyboard editor can write to it.
    settings: Config,
    keys: keys::State,
    keys_visible: bool,
    /// The settings window's own state: which page, what is searched for,
    /// what the file complained about.
    settings_state: crate::ui::settings::State,
    settings_visible: bool,
    /// Whether the keyboard editor is the reason the viewer is deaf, so only
    /// it undoes that.
    muted_for_keys: bool,
    /// A fullscreen change asked for by a mode, sent on the next frame.
    pending_fullscreen: Option<bool>,
    /// Whether the window was already fullscreen before a mode asked for it,
    /// so leaving that mode puts it back the way the user had it.
    was_fullscreen: bool,
    /// What has gone wrong lately, on its way to the user.
    notices: Notices,
    /// Whether a mark moves on to the next photograph by itself.
    advancing: bool,
    /// A deletion the user has been asked about but has not answered.
    pending_delete: Option<cull::Pending>,
    /// A bulk undo that has been asked about but not answered, as the sentence
    /// saying what it would do.
    pending_undo: Option<String>,
    /// Where photographs were last sent, so the same key twice repeats it.
    last_destination: Option<Slot>,
    last_errand: Option<Errand>,
    /// The panel asking where they should go, while it is up.
    asking: Option<Asking>,
    /// How to put back whatever the last thing did.
    journal: Journal,
    /// How the folder is narrowed and ordered, and whether its bar is up.
    narrowing: Narrowing,
    /// The runs of frames the folder holds, when it is being shown stacked.
    stacking: Stacking,
    filter_visible: bool,
    /// What every photograph in the open collection carries, in the order
    /// `paths` holds them.
    ///
    /// Built once when the collection changes rather than asked per cell per
    /// frame: the contact sheet needs all of it at once, and reading a
    /// folder's sidecars is a few milliseconds one time.
    marks: Vec<Marks>,
    /// Which files follow which, when a raw and a JPEG are one photograph.
    ///
    /// The partner is not in `paths` and is never shown; every command that
    /// touches a file expands through this so that none of them has to know
    /// that pairing exists.
    pairs: Pairs,
    /// Where the last run left off, kept up to date and written on the way out.
    session: Session,
    /// Whether the strip of thumbnails is under the photograph.
    ///
    /// Starts on when the configured height says so, and the key toggles it
    /// for the session.
    filmstrip_visible: bool,
    /// Whether the sheet of keys is up.
    ///
    /// Not the editor, which is a settings window: this is the glance-at list
    /// of what the keys currently are.
    cheat_sheet_visible: bool,
    /// Set on the frame it was opened, so that frame's key does not close it.
    cheat_sheet_opened: bool,
    /// What is being searched for in the cheat sheet.
    cheat_sheet_query: String,
    /// The keywords this folder has been seen to use, and the revision of the
    /// annotations they were read at.
    ///
    /// `None` until the panel has been opened once. An empty list is a real
    /// answer — most folders have no keywords in them — so "not read yet" has
    /// to be something other than "read, and empty".
    seen_tags: (Option<u64>, Vec<String>),
    /// Whether the question about a configuration file edited underneath the
    /// viewer is on screen.
    conflict_visible: bool,
    /// Whether the first-run hint is still on screen.
    ///
    /// Dismissed by pressing either of the two keys it names, and not shown at
    /// all after the first session.
    hint_visible: bool,
    /// Whether this is the first run: there was no session file to read.
    ///
    /// `Session::load` hands back a default for a missing file and an
    /// unreadable one alike, so the file itself has to be looked for.
    first_session: bool,
    /// What this build is, read once when the window is made.
    about: about::About,
    about_visible: bool,
    legend_visible: bool,
    placeholders_visible: bool,
    messages_visible: bool,
    /// A theme change asked for by the settings window, applied on the next
    /// frame: `set_theme` inside a frame that has already begun is a frame
    /// drawn half in each.
    pending_theme: Option<bool>,
    /// Full size decodes on their way to the clipboard.
    copying: verbs::Copying,
    /// Text waiting to go on the clipboard, which needs a context to do.
    pending_clipboard: Option<String>,
    /// Commands raised from a place with no egui context to hand, run at the
    /// end of the frame that raised them.
    pending_commands: Vec<Command>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        slideshow: bool,
        fullscreen: bool,
        benchmark: bool,
    ) -> App {
        theme::apply_theme(&cc.egui_ctx, config.general.theme == "light");
        crate::annotations::sidecar::name_like_adobe(config.tags.sidecar_naming == "replacing");
        apply_text_scaling(&cc.egui_ctx, config.general.text_scaling);

        if fullscreen {
            cc.egui_ctx
                .send_viewport_cmd(ViewportCommand::Fullscreen(true));
        }

        let render_state = cc
            .wgpu_render_state
            .clone()
            .expect("avis-imgv requires the wgpu backend");

        let loader = Arc::new(Loader::new(config.cache.decode_threads));
        let output_profile: Arc<str> = Arc::from(config.general.output_icc_profile.as_str());
        let image_budget = stores::image_store(&config.cache, &config.image_view, &config.raw);
        let thumbnail_budget = stores::thumbnail_store(&config.cache, &config.grid_view);

        let mut opening = crawler::paths_from_args();
        crawler::sort(&mut opening.images);

        // Kept whole so the keyboard editor has something to write back; the
        // views take their own copies of the parts they need.
        let settings = config.clone();
        // A first run is one with no session file. `Session::load` hands back
        // a default for a missing file and an unreadable one alike, so the
        // file itself has to be looked for.
        let first_session = Session::path().is_none_or(|path| !path.exists());
        let session = Session::load();

        // Read before the configuration is handed round, and the only thing
        // wanted from it here.
        // Its own field now. It used to be derived from the height, which is
        // why the key that shows the strip did nothing on a fresh install: the
        // default height is zero.
        let filmstrip =
            settings.grid_view.filmstrip_visible || settings.general.panels_at_start.filmstrip;
        let advancing = config.tags.advance_after_marking;
        let panels = config.general.panels_at_start;

        // Read here because this is where the adapter is: it was told to the
        // log at startup and to nothing a person can see.
        let about = about::About {
            version: env!("CARGO_PKG_VERSION"),
            adapter: describe_adapter(&render_state),
            libraw: crate::decoder::raw::version(),
        };

        let mut app = App {
            image_view: ImageView::new(
                render_state.clone(),
                Arc::clone(&loader),
                image_budget,
                Arc::clone(&output_profile),
                config.image_view,
                config.slideshow,
                slideshow,
            ),
            grid_view: GridView::new(
                render_state,
                loader,
                thumbnail_budget,
                output_profile,
                config.grid_view,
            ),
            base_path: base_path_of(&opening.images),
            navigator_path: String::new(),
            paths: Vec::new(),
            flattened: false,
            organize_view: OrganizeView::new(&config.group),
            mode: Mode::default(),
            menu_visible: panels.menu || first_session || session.menu_visible,
            side_panel_visible: panels.side_panel,
            metrics_visible: false,
            overlay: None,
            watcher: watcher::DirectoryWatcher::default(),
            perf_metrics: PerfMetrics::new(),
            config: config.general,
            annotations: AnnotationStore::new(),
            catalog: Catalog::configured(&config.tags),
            recent_tags: RecentTags::load(config.tags.recent_tags),
            tag_panel: tag_panel::State::default(),
            tag_panel_visible: panels.tag_panel,
            tag_config: config.tags,
            benchmark: benchmark.then(|| Benchmark::new(BENCHMARK_IMAGES)),
            settings,
            keys: keys::State::default(),
            keys_visible: false,
            settings_state: crate::ui::settings::State::default(),
            settings_visible: false,
            muted_for_keys: false,
            pending_fullscreen: None,
            was_fullscreen: fullscreen,
            notices: Notices::default(),
            advancing,
            pending_delete: None,
            pending_undo: None,
            last_destination: None,
            last_errand: None,
            asking: None,
            journal: Journal::default(),
            narrowing: Narrowing::of(&config.browsing),
            stacking: Stacking::of(&config.group, config.browsing.stack_by_default),
            filter_visible: false,
            marks: Vec::new(),
            pairs: Pairs::default(),
            session,
            filmstrip_visible: filmstrip,
            cheat_sheet_visible: false,
            cheat_sheet_opened: false,
            cheat_sheet_query: String::new(),
            seen_tags: (None, Vec::new()),
            conflict_visible: false,
            first_session,
            hint_visible: first_session,
            about,
            about_visible: false,
            legend_visible: false,
            placeholders_visible: false,
            messages_visible: false,
            pending_theme: None,
            copying: verbs::Copying::default(),
            pending_clipboard: None,
            pending_commands: Vec::new(),
        };

        for clash in keys::clashes(&app.settings) {
            app.notices.warn(clash);
        }

        // A first run has just written a configuration file somewhere the
        // person has never looked, and the README gives the wrong place on two
        // of the three platforms.
        if first_session {
            if let Some(path) = Config::path() {
                app.notices
                    .say(format!("Settings are kept in {}", path.display()));
            }
        }

        for said in &app.settings.migrated {
            app.notices.say(format!("Brought forward: {said}"));
        }

        if app.settings.partial {
            app.notices.fail(
                "Part of the configuration file could not be read; those settings are \
                 at their defaults and the file is not being written over",
            );
        }

        // Where the command line said, or where the last run left off when it
        // said nothing. A path that was typed always wins: somebody who named a
        // folder meant that folder. With nothing named the working directory is
        // what was read, and the working directory of a viewer started from a
        // desktop icon is nobody's choice.
        let restoring = app.config.restore_session && !opening.named;

        if let Some(folder) = app.session.folder.clone().filter(|_| restoring) {
            tracing::info!("Restoring the last session in {}", folder.display());
            let landing = app.session.position_in(&folder).map(Path::to_path_buf);
            app.open_directory(&folder, landing.as_deref());
        } else {
            let folder = opening.folder.clone();
            let landing = opening
                .selected
                .clone()
                // Nothing named, but this folder has been culled before: pick
                // up where that left off rather than at the first frame.
                .or_else(|| {
                    let folder = folder.as_deref()?;
                    app.config
                        .restore_session
                        .then(|| app.session.position_in(folder).map(Path::to_path_buf))?
                });

            app.open_within(
                std::mem::take(&mut opening.images),
                landing.as_deref(),
                opening.folder,
            );
        }

        app
    }

    /// Opens a collection, remembering which folder it came from.
    ///
    /// `folder` is the one that was actually asked for. Without it the folder
    /// has to be guessed from the first photograph, and a folder holding no
    /// photographs at all leaves nothing to guess from — so opening an empty
    /// one used to quietly put the viewer in the *home* directory, and asking
    /// to flatten it then crawled everything the user owns.
    fn open_within(
        &mut self,
        paths: Vec<PathBuf>,
        selected: Option<&Path>,
        folder: Option<PathBuf>,
    ) {
        let arriving = folder.unwrap_or_else(|| base_path_of(&paths));

        // A raw and a JPEG of the same frame are one photograph: one of them
        // is browsed and the other follows it through every command.
        let (paths, pairs) = Pairs::gather(&paths, self.settings.raw.pair_with_jpeg);
        self.pairs = pairs;

        // The watcher follows the folder that is open. It used to stay on the
        // one it was started on, so walking away from a watched folder left it
        // reporting arrivals somewhere nobody was looking — and reporting
        // nothing about the folder actually on screen, while the status bar
        // said "Watching".
        if self.watcher.is_active() && arriving != self.base_path {
            self.watcher.restart(&arriving, self.flattened);
        }

        self.base_path = arriving;
        self.navigator_path = self.base_path.to_string_lossy().to_string();
        self.paths = paths;
        self.marks.clear();

        // The partner of a paired photograph is not in the collection, so
        // landing on one means landing on the half that is browsed.
        let selected = selected.map(|path| self.browsed(path));

        self.image_view
            .set_images(self.paths.clone(), selected.as_deref());
        self.grid_view.set_images(self.paths.clone());

        // A different folder is a different set of runs, and the frames the
        // last one was stacked into mean nothing here.
        self.stacking.reopen(&self.paths);

        // Only when there is something to apply: narrowing reads the sidecar
        // of every file in the folder, and opening one should not pay for
        // that when no rule is on.
        if !self.narrowing.is_idle() || self.stacking.is_on() {
            self.apply_narrowing();
        }
    }

    /// Whether the mark cache currently mirrors the collection.
    ///
    /// It does not always: it is filled only when the contact sheet is about
    /// to draw, so until then it is empty however many photographs are open.
    /// Anything that keeps it in step with a change to the collection has to
    /// ask first — inserting into or removing from a vector by a position it
    /// does not have is a panic, and "the folder changed before the sheet was
    /// ever opened" is the ordinary case rather than the exotic one.
    fn marks_are_in_step(&self) -> bool {
        self.marks.len() == self.paths.len()
    }

    /// Takes a photograph's marks out, if they are being kept.
    ///
    /// When they are not, the shorter list is left alone: [`App::ensure_marks`]
    /// reads the whole collection again the next time the sheet needs it,
    /// which is exactly what a length that does not match asks it to do.
    pub(super) fn drop_mark(&mut self, index: usize) {
        if self.marks_are_in_step() {
            self.marks.remove(index);
        }
    }

    /// Puts a photograph's marks in at `index`, if they are being kept.
    pub(super) fn add_mark(&mut self, index: usize, image: &Path) {
        if !self.marks_are_in_step() {
            return;
        }

        let marks = Marks::of(self.annotations.get(image, None));
        self.marks.insert(index, marks);
    }

    /// The half of a pair that is browsed, for a path that might be either.
    ///
    /// Opening a folder on a named file has to land on the photograph that is
    /// actually in the collection: naming the raw of a pair whose JPEG is
    /// browsed would otherwise land on nothing.
    fn browsed(&self, path: &Path) -> PathBuf {
        if self.paths.iter().any(|shown| shown == path) {
            return path.to_path_buf();
        }

        self.paths
            .iter()
            .find(|shown| self.pairs.partners_of(shown).iter().any(|p| p == path))
            .cloned()
            .unwrap_or_else(|| path.to_path_buf())
    }

    /// Every file in the open collection, browsed or following.
    pub(super) fn all_paths(&self) -> Vec<PathBuf> {
        self.pairs.everything(&self.paths)
    }

    /// Every file a command about `path` should touch: it and its partner.
    pub(super) fn with_partners(&self, path: &Path) -> Vec<PathBuf> {
        self.pairs.with_partners(path)
    }

    /// Reads the marks for the whole collection, if that has not been done.
    ///
    /// Only called when the contact sheet is about to draw, because it is the
    /// only thing that needs all of them; the image view asks about one at a
    /// time and the sidecar for that one is already read.
    fn ensure_marks(&mut self) {
        if self.marks.len() == self.paths.len() {
            return;
        }

        let paths = self.paths.clone();
        self.marks = paths
            .iter()
            .map(|path| Marks::of(self.annotations.get(path, None)))
            .collect();
    }

    /// Recomputes what is on show and hands it to both views.
    ///
    /// Cheap on purpose: a vector of positions rather than a new collection,
    /// so applying a filter in the middle of a cull does not throw away the
    /// decoded folder. Called whenever a rule changes and whenever a mark
    /// changes something a rule depends on, which is what makes rejecting a
    /// frame with the rejects hidden remove it from the strip at once.
    fn apply_narrowing(&mut self) {
        self.ensure_marks();

        let visible = self.narrowing.apply(&self.paths, &self.marks);

        // Stacking after narrowing, not instead of it: a filter says which
        // photographs are worth looking at and a stack says how many of them
        // are the same photograph, and the second question only makes sense
        // over the answer to the first.
        let visible = self.stacking.fold(visible, self.paths.len());

        self.image_view.set_visible(visible.clone());
        self.grid_view.set_visible(visible);
    }

    /// Keeps the stacks up to date while the folder is being read.
    ///
    /// The scan arrives in batches, so the sheet folds up as the reading gets
    /// to each run rather than after a wait in front of a still screen; while
    /// it is going the frame is asked for again, because nothing else on
    /// screen is changing to ask for it.
    fn handle_stacking(&mut self, ctx: &egui::Context) {
        if self.stacking.poll(&self.paths) {
            self.apply_narrowing();
        }

        if self.stacking.progress().is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
        }
    }

    /// Shows the folder stacked, or puts every frame back.
    fn toggle_stacking(&mut self) {
        if self.stacking.is_on() {
            self.stacking.turn_off();
            self.notices.say("Stacks off");
        } else {
            self.stacking.turn_on(&self.paths);
            self.notices.say("Reading the folder for stacks");
        }

        self.apply_narrowing();
    }

    /// Opens or closes the stack the cursor is in.
    ///
    /// Turning stacking on first when it is off: a key that does nothing at
    /// all is worse than one that does the obvious thing, and the obvious
    /// thing to do with a stack key on an unstacked folder is to stack it.
    fn toggle_stack(&mut self) {
        if !self.stacking.is_on() {
            self.toggle_stacking();
            return;
        }

        let Some(index) = self.cursor_index() else {
            return;
        };

        // The frame standing for a stack that opens should stay the frame on
        // screen, so opening one lands where the eye already was.
        if self.stacking.toggle(index) {
            self.apply_narrowing();
            self.go_to(index);
        }
    }

    /// Changes which frame stands for the stack the cursor is in.
    fn step_standing(&mut self, forward: bool) {
        let Some(index) = self.cursor_index() else {
            return;
        };

        if let Some(standing) = self.stacking.step_standing(index, forward) {
            self.apply_narrowing();
            self.go_to(standing);
        }
    }

    /// Steps to the next run of frames, or the one before it.
    fn step_stack(&mut self, forward: bool) {
        let Some(index) = self.cursor_index() else {
            return;
        };

        if let Some(landing) = self.stacking.stacks().step_stack(index, forward) {
            self.go_to(landing);
        }
    }

    /// The store position the keyboard is on, in whichever view is up.
    fn cursor_index(&self) -> Option<usize> {
        match self.mode {
            Mode::Grid => self.grid_view.cursor(),
            _ => Some(self.image_view.selected_index()).filter(|at| *at < self.paths.len()),
        }
    }

    /// Puts both views on a store position.
    fn go_to(&mut self, index: usize) {
        let Some(path) = self.paths.get(index).cloned() else {
            return;
        };

        self.image_view.select_path(&path);
        self.grid_view.focus_on(index);
    }

    /// Draws the filter bar and applies whatever it changed.
    fn show_filter_bar(&mut self, ctx: &egui::Context) {
        let shown = match self.mode {
            Mode::Grid => self.grid_view.position().1,
            _ => self.image_view.position().1,
        };

        // Read out and handed back rather than borrowed: the bar is drawn
        // with the whole application borrowed already, and the detector cannot
        // be asked to read the folder again from inside the closure that is
        // drawing it.
        let mut settings = self.stacking.settings();
        let mut state = filter_bar::StackState {
            on: self.stacking.is_on(),
            found: self.stacking.stacks().len(),
            stacked: self.stacking.stacks().stacked(),
            all_collapsed: self.stacking.stacks().all_collapsed(),
            reading: self.stacking.progress(),
            settings: &mut settings,
        };

        let (changed, stacked) = filter_bar::ui(
            ctx,
            self.filter_visible,
            &mut self.narrowing,
            (shown, self.paths.len()),
            &mut state,
        );

        if stacked.toggled {
            self.toggle_stacking();
        }

        if stacked.retuned && self.stacking.retune(settings, &self.paths) {
            self.apply_narrowing();
        }

        if let Some(collapsed) = stacked.set_all {
            self.stacking.set_all(collapsed);
            self.apply_narrowing();
        }

        if changed {
            self.apply_narrowing();
        }
    }

    /// Records where the viewer is, so the next run can start here.
    ///
    /// Called every frame and costs a comparison: the session is written on the
    /// way out, not on every step through a folder.
    fn note_position(&mut self, ctx: &egui::Context) {
        if !self.config.restore_session {
            return;
        }

        self.note_window(ctx);

        let showing = self.marked_path();
        let already = self.session.position_in(&self.base_path);

        if already == showing.as_deref() && self.session.folder.as_deref() == Some(&self.base_path)
        {
            return;
        }

        let base = self.base_path.clone();
        self.session.remember(&base, showing.as_deref());
    }

    /// Records the chrome that is remembered between runs.
    ///
    /// Cheap, and outside the early return above, because the menu bar is not a
    /// position: it is what the person left the window looking like.
    fn note_chrome(&mut self) {
        self.session.menu_visible = self.menu_visible;
    }

    /// Where the window is, as the platform reports it this frame.
    ///
    /// Taken while running rather than on the way out, because by the time the
    /// viewer is closing the window may already have been given up — and a
    /// maximised window reports the size it would be if it were not, which is
    /// the size worth restoring alongside the maximised flag.
    fn note_window(&mut self, ctx: &egui::Context) {
        let found = ctx.input(|i| {
            let viewport = i.viewport();

            viewport.inner_rect.map(|rect| Geometry {
                width: rect.width(),
                height: rect.height(),
                x: Some(rect.min.x),
                y: Some(rect.min.y),
                maximised: viewport.maximized.unwrap_or(false),
            })
        });

        // A window being dragged reports a new place every frame, and none of
        // them is worth writing until the viewer closes — so this only keeps
        // the latest, and the file is written once.
        if let Some(found) = found.filter(Geometry::is_usable) {
            self.session.window = Some(found);
        }
    }

    /// Brings a whole batch's marks up to date, and re-narrows once.
    ///
    /// Once rather than per photograph: applying a filter is a pass over the
    /// collection, and doing that two hundred times because two hundred frames
    /// were rated at once is the difference between instant and a visible
    /// stall.
    pub(super) fn refresh_marks(&mut self, images: &[PathBuf]) {
        for image in images {
            self.recompute_mark(image);
        }

        if !self.narrowing.is_idle() {
            self.apply_narrowing();
        }
    }

    /// Brings one photograph's marks up to date after it has been changed.
    pub(super) fn refresh_mark(&mut self, image: &Path) {
        self.recompute_mark(image);

        // A filter that hides the rejects has to hide one the moment it is
        // rejected, or the mark appears not to have taken.
        if !self.narrowing.is_idle() {
            self.apply_narrowing();
        }
    }

    /// Reads one photograph's marks back out of the store, without re-filtering.
    fn recompute_mark(&mut self, image: &Path) {
        let Some(index) = self.paths.iter().position(|path| path == image) else {
            return;
        };

        let Some(found) = self.marks.get_mut(index) else {
            return;
        };

        *found = self
            .annotations
            .peek(image)
            .map(Marks::of)
            .unwrap_or_default();
    }

    /// Crawls `path` and opens what it finds.
    fn open_directory(&mut self, path: &Path, selected: Option<&Path>) {
        let mut paths = crawler::crawl(path, self.flattened);
        crawler::sort(&mut paths);
        self.open_within(paths, selected, Some(path.to_path_buf()));
    }

    pub(super) fn apply(&mut self, command: Command, ctx: &egui::Context) {
        match command {
            Command::Exit => ctx.send_viewport_cmd(ViewportCommand::Close),
            Command::ToggleGrid => {
                self.set_mode(match self.mode {
                    Mode::Grid => Mode::Image,
                    _ => Mode::Grid,
                });
            }
            Command::NextMode => self.set_mode(self.mode.next()),
            Command::SetMode(mode) => self.set_mode(mode),
            Command::ToggleMenu => self.menu_visible = !self.menu_visible,
            Command::ToggleSidePanel => self.side_panel_visible = !self.side_panel_visible,
            Command::ToggleMetrics => self.metrics_visible = !self.metrics_visible,
            Command::ToggleFlatten => self.toggle_flatten(),
            Command::ToggleWatcher => {
                self.watcher.toggle(&self.base_path.clone(), self.flattened);
            }
            Command::ToggleTagPanel => self.tag_panel_visible = !self.tag_panel_visible,
            Command::SetRating(stars) => self.rate(stars),
            Command::SetFlag(flag) => self.flag(flag),
            Command::SetLabel(index) => self.label(index),
            Command::ToggleAdvance => self.advancing = !self.advancing,
            Command::Delete => self.delete_open_image(false),
            Command::DeletePermanently => self.delete_open_image(true),
            Command::MoveTo => self.send_somewhere(Errand::Move),
            Command::CopyTo => self.send_somewhere(Errand::Copy),
            Command::ToRejectedFolder => self.send_to_rejected(),
            Command::Undo => self.undo(),
            Command::ToggleFilmstrip => self.filmstrip_visible = !self.filmstrip_visible,
            Command::ToggleStacking => self.toggle_stacking(),
            Command::ToggleStack => self.toggle_stack(),
            Command::StandingBack => self.step_standing(false),
            Command::StandingForward => self.step_standing(true),
            Command::PreviousStack => self.step_stack(false),
            Command::NextStack => self.step_stack(true),
            Command::ShowSettings => self.open_settings(),
            Command::ShowKeys => {
                self.cheat_sheet_visible = !self.cheat_sheet_visible;
                // The key that opened it is still going down this frame, and
                // any key closes it.
                self.cheat_sheet_opened = self.cheat_sheet_visible;
            }
            Command::ToggleFilter => {
                self.filter_visible = !self.filter_visible;
            }
            Command::SuspendFilter => {
                self.narrowing.suspended = !self.narrowing.suspended;
                self.apply_narrowing();
            }
            Command::ToggleFullscreen => {
                let wanted = !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                self.was_fullscreen = wanted;
                self.pending_fullscreen = Some(wanted);
            }
        }
    }

    /// Switches what the window is for.
    ///
    /// Entering a folder job reads the folder, which is why it happens here
    /// rather than every frame: the sweep is only worth starting when the mode
    /// is actually opened, and only when it is not already holding the folder.
    /// Folds sub-directories into the collection, or unfolds them again.
    fn toggle_flatten(&mut self) {
        self.flattened = !self.flattened;
        tracing::info!("Flattened directories: {}", self.flattened);

        let base = self.base_path.clone();
        let selected = self.image_view.active_path();

        self.watcher.restart(&base, self.flattened);
        self.open_directory(&base, selected.as_deref());
    }

    /// Reads the open folder again, keeping the photograph on screen.
    ///
    /// What pairing changes is what the collection *is*, so the answer to a
    /// change of `raw.pair_with_jpeg` is a re-read rather than a restart.
    pub(super) fn reopen_folder(&mut self) {
        let base = self.base_path.clone();
        let selected = self.image_view.active_path();

        self.open_directory(&base, selected.as_deref());
    }

    /// Runs one command from somewhere that has no context to hand.
    pub(super) fn apply_command(&mut self, command: Command) {
        self.pending_commands.push(command);
    }

    /// Moves anything that failed on a background thread onto the screen.
    fn report_problems(&mut self) {
        for problem in self.annotations.problems() {
            self.notices.say(problem);
        }
    }

    fn execute_callback(&mut self, callback: Callback) {
        tracing::info!("Executing callback {callback:?}");

        match callback {
            Callback::Pop(Some(path)) => {
                self.paths.retain(|candidate| candidate != &path);
                self.image_view.pop(&path);
                self.grid_view.pop(&path);
            }
            Callback::Reload(Some(path)) => {
                self.image_view.reload(&path);
                self.grid_view.reload(&path);
                self.annotations.forget(&path);
            }
            Callback::ReloadAll => {
                let base = self.base_path.clone();
                let selected = self.image_view.active_path();
                self.open_directory(&base, selected.as_deref());
            }
            Callback::Advance => self.image_view.next_image(),
            Callback::Pop(None) | Callback::Reload(None) | Callback::NoAction => {}
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.perf_metrics.new_frame();

        // Before anything is requested, so the first decodes are already the
        // right size rather than a screenful of wasted megapixels.
        self.image_view
            .set_display_edge(longest_edge_in_pixels(ctx));

        // Wherever the keyboard has wandered off to, Escape brings it back.
        // A text field with focus mutes every shortcut in the viewer, and
        // finding out which field has it is not the user's job.
        if self.overlay.is_none() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            crate::utils::surrender_focus(ctx);
        }

        input::update_overlay(ctx, &mut self.overlay, &self.config);
        for command in input::collect(ctx, &self.config, &self.tag_config, &self.settings.cull) {
            // Marking a selection never advances: the mark went to two
            // hundred photographs rather than to the one on screen, so there
            // is nothing for "the next one" to mean.
            let advance =
                input::advances(command, self.advancing) && self.grid_view.selected_count() == 0;
            self.apply(command, ctx);

            // After the mark, not before it: the mark belongs to the
            // photograph that was on screen when the key went down.
            if advance {
                self.image_view.next_image();
            }
        }

        egui::TopBottomPanel::top("performance_metrics")
            .show_separator_line(false)
            .show_animated(ctx, self.metrics_visible, |ui| {
                self.perf_metrics.display_metrics(ui)
            });

        let menu_keys = panels::MenuKeys {
            cheat_sheet: "?".to_string(),
            settings: keys::describe(&self.config.sc_settings),
        };
        if let Some(action) = panels::top_menu(ctx, self.menu_visible, self.mode, &menu_keys) {
            self.handle_menu(action);
        }

        if self.cheat_sheet_visible {
            let just_opened = std::mem::take(&mut self.cheat_sheet_opened);
            self.cheat_sheet_visible = cheat_sheet::ui(
                ctx,
                &self.settings,
                self.mode,
                just_opened,
                &mut self.cheat_sheet_query,
            );
        }

        self.show_first_run_hint(ctx);
        panels::typing_notice(ctx);
        self.show_filter_bar(ctx);
        self.show_destinations(ctx);
        self.show_pending_delete(ctx);
        self.show_pending_undo(ctx);
        self.show_keyboard(ctx);
        self.show_settings(ctx);
        self.show_conflict(ctx);
        self.show_help_windows(ctx);
        self.apply_fullscreen(ctx);
        self.show_side_panel(ctx);
        self.show_tag_panel(ctx);
        self.show_overlays(ctx);
        self.show_views(ctx);
        self.handle_watcher();
        self.handle_stacking(ctx);

        if self.watcher.is_active() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }

        if let Some(light) = self.pending_theme.take() {
            theme::apply_theme(ctx, light);
            apply_text_scaling(ctx, self.config.text_scaling);
        }

        self.handle_pending_commands(ctx);
        self.handle_copying(ctx);
        self.report_problems();
        if self.notices.ui(ctx) {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        self.note_position(ctx);
        self.note_chrome();
        self.run_benchmark(ctx);
        self.perf_metrics.end_frame();
    }

    /// Written on the way out: the window as it stands, and where the viewer
    /// had got to in every folder it visited.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.config.restore_session {
            return;
        }

        self.session.save();
    }
}

/// Which graphics adapter the photographs are being drawn on.
///
/// Said out loud because "the viewer is slow" and "the colours are wrong" are
/// both questions whose answer often starts here, and the answer reached only
/// a log file whose own path was written into that same log.
fn describe_adapter(render_state: &eframe::egui_wgpu::RenderState) -> String {
    let info = render_state.adapter.get_info();

    format!("{} — {:?}, {:?}", info.name, info.device_type, info.backend)
}

/// The most pixels an image could ever be shown across on this screen.
///
/// The monitor rather than the window, because the answer travels with every
/// decode request: sizing to the window would mean re-decoding a folder every
/// time one is dragged bigger, and maximising is exactly when a viewer must
/// not stutter. Falls back to the window where the monitor is not known.
fn longest_edge_in_pixels(ctx: &egui::Context) -> u32 {
    let monitor = ctx.input(|input| input.viewport().monitor_size);
    let size = monitor.unwrap_or_else(|| ctx.content_rect().size()) * ctx.pixels_per_point();

    size.x.max(size.y).max(1.0) as u32
}

/// Scales every text style from the one the theme set, not from the last one.
///
/// Two things had to be true before the field could be changed while the
/// window is open. Zero multiplies every style to nothing, including the menu
/// bar that would let anybody undo it, so it is floored; and multiplying the
/// *current* style compounds — 1.25 applied twice is 1.5625 — so the style the
/// theme built is kept and each call starts from it.
fn apply_text_scaling(ctx: &egui::Context, scaling: f32) {
    let scaling = scaling.clamp(MIN_TEXT_SCALING, MAX_TEXT_SCALING);

    // Read before the store is locked. `Context::style` takes the same lock
    // `data_mut` holds, so asking for it inside the closure hangs the viewer
    // before it has drawn a frame.
    let mut style = (*ctx.style()).clone();
    let current = BaseTextStyles(style.text_styles.clone());

    let base = ctx.data_mut(|data| {
        data.get_temp_mut_or_insert_with(egui::Id::new("text scaling base"), || current)
            .clone()
    });

    style.text_styles = base.0;
    for font in style.text_styles.values_mut() {
        font.size *= scaling;
    }
    ctx.set_style(style);
}

/// The sizes the theme asked for, before any scaling was applied to them.
#[derive(Clone)]
struct BaseTextStyles(std::collections::BTreeMap<egui::TextStyle, egui::FontId>);

/// Half size. Below this the menu bar cannot be read, and the menu bar is the
/// way back.
pub const MIN_TEXT_SCALING: f32 = 0.5;

/// Three times. Past this a single row of the interface fills the window.
pub const MAX_TEXT_SCALING: f32 = 3.0;

/// The directory the open images live in, falling back to the user's home.
fn base_path_of(paths: &[PathBuf]) -> PathBuf {
    if let Some(parent) = paths.first().and_then(|path| path.parent()) {
        return parent.to_path_buf();
    }

    directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The case that sent the viewer home: nothing to take a parent from.
    #[test]
    fn an_empty_collection_has_no_folder_to_derive() {
        assert_ne!(base_path_of(&[]), PathBuf::new());
    }

    #[test]
    fn the_base_path_is_the_parent_of_the_first_image() {
        let paths = vec![PathBuf::from("/photos/trip/a.jpg")];
        assert_eq!(base_path_of(&paths), PathBuf::from("/photos/trip"));
    }

    #[test]
    fn an_empty_collection_falls_back_to_a_real_directory() {
        assert!(
            !base_path_of(&[]).as_os_str().is_empty() || directories::UserDirs::new().is_none()
        );
    }
}

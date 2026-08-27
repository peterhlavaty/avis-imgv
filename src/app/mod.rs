//! The application: which folder is open, which view shows it, and the wiring
//! between them.

pub mod input;
pub mod panels;
pub mod stores;
pub mod tagging;
pub mod watcher;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, ViewportCommand};

use crate::actions::Callback;
use crate::annotations::{AnnotationStore, Catalog, RecentTags};
use crate::cache::loader::Loader;
use crate::config::{Config, GeneralConfig, TagConfig};
use crate::crawler;
use crate::formats;
use crate::ui::tag_panel;
use crate::ui::{navigator, perf_metrics::PerfMetrics, theme, tree};
use crate::view::image_view::bottom_bar::Flags;
use crate::view::{GridView, ImageView};

use input::{Command, Overlay};
use panels::MenuAction;

pub struct App {
    image_view: ImageView,
    grid_view: GridView,
    /// The images currently open, before either view partitions them.
    paths: Vec<PathBuf>,
    base_path: PathBuf,
    /// Whether sub-directories are folded into the open collection.
    flattened: bool,
    grid_visible: bool,
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
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, slideshow: bool, fullscreen: bool) -> App {
        let config = Config::new();

        theme::apply_theme(&cc.egui_ctx);
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

        let (mut paths, opened) = crawler::paths_from_args();
        paths.sort();

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
            base_path: base_path_of(&paths),
            navigator_path: String::new(),
            paths: Vec::new(),
            flattened: false,
            grid_visible: false,
            menu_visible: false,
            side_panel_visible: false,
            metrics_visible: false,
            overlay: None,
            watcher: watcher::DirectoryWatcher::default(),
            perf_metrics: PerfMetrics::new(),
            config: config.general,
            annotations: AnnotationStore::new(),
            catalog: Catalog::new(config.tags.categories.clone()),
            recent_tags: RecentTags::load(config.tags.recent_tags),
            tag_panel: tag_panel::State::default(),
            tag_panel_visible: false,
            tag_config: config.tags,
        };

        app.open(paths, opened.as_deref());
        app
    }

    /// Opens a set of images, optionally landing on one of them.
    fn open(&mut self, paths: Vec<PathBuf>, selected: Option<&Path>) {
        self.base_path = base_path_of(&paths);
        self.navigator_path = self.base_path.to_string_lossy().to_string();
        self.paths = paths;

        self.image_view.set_images(self.paths.clone(), selected);
        self.grid_view.set_images(self.paths.clone());
    }

    /// Crawls `path` and opens what it finds.
    fn open_directory(&mut self, path: &Path, selected: Option<&Path>) {
        let mut paths = crawler::crawl(path, self.flattened);
        paths.sort();
        self.open(paths, selected);
    }

    fn apply(&mut self, command: Command, ctx: &egui::Context) {
        match command {
            Command::Exit => ctx.send_viewport_cmd(ViewportCommand::Close),
            Command::ToggleGrid => self.grid_visible = !self.grid_visible,
            Command::ToggleMenu => self.menu_visible = !self.menu_visible,
            Command::ToggleSidePanel => self.side_panel_visible = !self.side_panel_visible,
            Command::ToggleMetrics => self.metrics_visible = !self.metrics_visible,
            Command::ToggleFlatten => self.toggle_flatten(),
            Command::ToggleWatcher => {
                self.watcher.toggle(&self.base_path.clone(), self.flattened);
            }
            Command::ToggleTagPanel => self.tag_panel_visible = !self.tag_panel_visible,
            Command::SetRating(stars) => self.rate(stars),
        }
    }

    /// Folds sub-directories into the collection, or unfolds them again.
    fn toggle_flatten(&mut self) {
        self.flattened = !self.flattened;
        tracing::info!("Flattened directories: {}", self.flattened);

        let base = self.base_path.clone();
        let selected = self.image_view.active_path();

        self.watcher.restart(&base, self.flattened);
        self.open_directory(&base, selected.as_deref());
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

    fn handle_menu(&mut self, action: MenuAction) {
        let dialog = rfd::FileDialog::new().set_directory(&self.base_path);

        match action {
            MenuAction::OpenFolder => {
                if let Some(folder) = dialog.pick_folder() {
                    self.open_directory(&folder, None);
                }
            }
            MenuAction::OpenFiles => {
                let picked = dialog
                    .add_filter("image", &formats::supported_extensions())
                    .pick_files();

                if let Some(files) = picked {
                    let first = files.first().cloned();
                    if let Some(parent) = first.as_ref().and_then(|f| f.parent()) {
                        self.open_directory(parent, first.as_deref());
                    }
                }
            }
        }
    }

    /// Applies files that appeared or changed while the watcher was on.
    fn handle_watcher(&mut self) {
        if !self.watcher.is_active() {
            return;
        }

        let known: Vec<PathBuf> = self.paths.clone();
        let changes = self
            .watcher
            .take_changes(|path| known.iter().any(|candidate| candidate == path));

        if changes.is_empty() {
            return;
        }

        for path in &changes.modified {
            self.image_view.reload(path);
            self.grid_view.reload(path);
            self.annotations.forget(path);
        }

        if !changes.added.is_empty() {
            let newest = changes.added.last().cloned();
            self.paths.extend(changes.added);
            self.paths.sort();

            let paths = self.paths.clone();
            self.image_view.set_images(paths.clone(), newest.as_deref());
            self.grid_view.set_images(paths);
        }
    }

    fn show_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("image_metadata")
            .resizable(true)
            .show_separator_line(false)
            .min_width(220.)
            .show_animated(ctx, self.side_panel_visible, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    panels::metadata_panel(
                        ui,
                        self.image_view.active_metadata(),
                        &self.config.metadata_tags,
                    );
                    ui.add_space(20.);
                    ui.separator();
                    panels::cache_stats(ui, &self.image_view.stats(), &self.grid_view.stats());
                });
            });
    }

    /// Draws the navigator and directory tree overlays.
    fn show_overlays(&mut self, ctx: &egui::Context) {
        match self.overlay {
            Some(Overlay::Navigator) => {
                if navigator::ui(&mut self.navigator_path, ctx) {
                    let path = PathBuf::from(self.navigator_path.clone());
                    input::close(ctx, &mut self.overlay);
                    self.open_directory(&path, None);
                }
            }
            Some(Overlay::DirectoryTree) => {
                let opened = self
                    .image_view
                    .active_path()
                    .unwrap_or_else(|| self.base_path.clone());

                if let Some(path) = tree::ui(&opened.to_string_lossy(), ctx) {
                    input::close(ctx, &mut self.overlay);
                    self.open_directory(&path, None);
                }
            }
            None => {}
        }
    }

    /// Draws whichever view is on screen, and keeps the other one's caches
    /// filling in behind it.
    fn show_views(&mut self, ctx: &egui::Context) {
        let warmed = if self.grid_visible {
            self.image_view.warm()
        } else {
            self.grid_view.warm(self.image_view.selected_index())
        };

        if warmed {
            ctx.request_repaint();
        }

        if self.grid_visible {
            self.grid_view
                .ui(ctx, Some(self.image_view.selected_index()));

            if let Some(path) = self.grid_view.take_selected() {
                self.image_view.select_path(&path);
                self.grid_visible = false;
            }

            if let Some(callback) = self.grid_view.take_callback() {
                self.execute_callback(callback);
            }

            return;
        }

        let rating = self
            .image_view
            .active_path()
            .and_then(|path| self.annotations.peek(&path).map(|found| found.rating))
            .unwrap_or(0);

        self.image_view.ui(
            ctx,
            Flags {
                flattened: self.flattened,
                watching: self.watcher.is_active(),
                ..Default::default()
            },
            rating,
        );

        if let Some(callback) = self.image_view.take_callback() {
            self.execute_callback(callback);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.perf_metrics.new_frame();

        input::update_overlay(ctx, &mut self.overlay, &self.config);
        for command in input::collect(ctx, &self.config, &self.tag_config) {
            self.apply(command, ctx);
        }

        egui::TopBottomPanel::top("performance_metrics")
            .show_separator_line(false)
            .show_animated(ctx, self.metrics_visible, |ui| {
                self.perf_metrics.display_metrics(ui)
            });

        if let Some(action) = panels::top_menu(ctx, self.menu_visible) {
            self.handle_menu(action);
        }

        self.show_side_panel(ctx);
        self.show_tag_panel(ctx);
        self.show_overlays(ctx);
        self.show_views(ctx);
        self.handle_watcher();

        if self.watcher.is_active() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }

        self.perf_metrics.end_frame();
    }
}

fn apply_text_scaling(ctx: &egui::Context, scaling: f32) {
    let mut style = (*ctx.style()).clone();
    for font in style.text_styles.values_mut() {
        font.size *= scaling;
    }
    ctx.set_style(style);
}

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

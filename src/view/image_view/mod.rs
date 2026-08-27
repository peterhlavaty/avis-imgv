//! The main view: one (or a few) images filling the window.

pub mod bottom_bar;
pub mod canvas;
pub mod input;
pub mod layout;
pub mod slideshow;
pub mod zoom;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Response};
use eframe::egui_wgpu::RenderState;
use eframe::epaint::{Color32, Vec2};

use crate::actions::{self, Callback};
use crate::cache::loader::Loader;
use crate::cache::{ImageState, ImageStore, StoreConfig, StoreStats};
use crate::config::{ImageViewConfig, SlideshowConfig};
use crate::metadata::Metadata;

use bottom_bar::{Flags, Status};
use canvas::{FrameStyle, Metrics, Viewport};
use input::Command;
use layout::BACKGROUND;
use slideshow::Slideshow;

/// Most images the view will place side by side. Beyond a handful they are too
/// small to read, and each one costs a texture.
const MAX_IMAGES_SHOWN: usize = 8;

pub struct ImageView {
    store: ImageStore,
    cursor: usize,
    viewport: Viewport,
    frame: FrameStyle,
    metrics: Metrics,
    images_shown: usize,
    jump_to: String,
    callback: Option<Callback>,
    config: ImageViewConfig,
    slideshow_config: SlideshowConfig,
    slideshow: Option<Slideshow>,
}

impl ImageView {
    pub fn new(
        render_state: RenderState,
        loader: Arc<Loader>,
        store_config: StoreConfig,
        output_profile: Arc<str>,
        config: ImageViewConfig,
        slideshow_config: SlideshowConfig,
        start_slideshow: bool,
    ) -> ImageView {
        let slideshow = start_slideshow.then(|| Slideshow::new(&slideshow_config));

        ImageView {
            store: ImageStore::new(render_state, loader, store_config, output_profile),
            cursor: 0,
            viewport: Viewport {
                // A slideshow always fills the screen.
                maximize: start_slideshow,
                ..Default::default()
            },
            frame: FrameStyle {
                enabled: start_slideshow && slideshow_config.start_with_frame_enabled,
                relative_size: config.frame_size_relative_to_image,
            },
            metrics: Metrics::default(),
            images_shown: config.nr_images_shown.clamp(1, MAX_IMAGES_SHOWN),
            jump_to: String::new(),
            callback: None,
            slideshow,
            slideshow_config,
            config,
        }
    }

    /// Opens a new collection, optionally starting on a specific image.
    pub fn set_images(&mut self, paths: Vec<PathBuf>, selected: Option<&Path>) {
        let selected = selected
            .and_then(|path| paths.iter().position(|candidate| candidate == path))
            .unwrap_or(0);

        self.store.set_paths(paths);
        self.select(selected);
    }

    pub fn selected_index(&self) -> usize {
        self.cursor
    }

    pub fn active_path(&self) -> Option<PathBuf> {
        self.store.path(self.cursor).map(Path::to_path_buf)
    }

    pub fn active_metadata(&self) -> Option<&Metadata> {
        self.store.metadata(self.cursor)
    }

    pub fn stats(&self) -> StoreStats {
        self.store.stats()
    }

    pub fn take_callback(&mut self) -> Option<Callback> {
        self.callback.take()
    }

    /// Moves to `index`, which is where the caches centre themselves.
    pub fn select(&mut self, index: usize) {
        let previous = self.cursor;
        self.cursor = if self.store.is_empty() {
            0
        } else {
            index.min(self.store.len() - 1)
        };

        self.store.set_cursor(self.cursor);

        if self.cursor != previous {
            self.viewport.reset_for_new_image();
            if let Some(slideshow) = &mut self.slideshow {
                slideshow.restart();
            }
        }
    }

    pub fn select_path(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.select(index);
        }
    }

    /// Removes an image from the collection, staying on the same position.
    pub fn pop(&mut self, path: &Path) {
        let Some(index) = self.store.index_of(path) else {
            return;
        };

        self.store.remove(index);
        self.select(self.cursor.min(self.store.len().saturating_sub(1)));
    }

    pub fn reload(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.store.reload(index);
        }
    }

    pub fn next_image(&mut self) {
        if self.store.is_empty() || self.should_wait() {
            return;
        }

        self.select((self.cursor + 1) % self.store.len());
    }

    pub fn previous_image(&mut self) {
        if self.store.is_empty() {
            return;
        }

        let last = self.store.len() - 1;
        self.select(if self.cursor == 0 {
            last
        } else {
            self.cursor - 1
        });
    }

    /// Services the caches without drawing, so the view is ready the moment it
    /// is shown again.
    pub fn warm(&mut self) -> bool {
        self.store.tick()
    }

    /// Draws the view and services its caches.
    pub fn ui(&mut self, ctx: &egui::Context, flags: Flags) {
        if self.warm() {
            ctx.request_repaint();
        }

        for command in input::collect(ctx, &self.config) {
            self.apply(command, ctx);
        }

        if self.slideshow.is_none() {
            self.show_bottom_bar(ctx, flags);
        }

        let response = self.show_images(ctx);
        self.handle_pointer(ctx, &response);
        self.handle_context_menu(&response);
        self.run_slideshow(ctx);
    }

    fn apply(&mut self, command: Command, ctx: &egui::Context) {
        match command {
            Command::Next => self.next_image(),
            Command::Previous => self.previous_image(),
            Command::Fit => zoom::fit(&mut self.viewport),
            Command::Fill => zoom::fill(&mut self.viewport, &self.metrics),
            Command::ToggleFillLatch => {
                self.viewport.maximize = !self.viewport.maximize;
                self.viewport.maximized = false;
            }
            Command::FitHorizontal => zoom::fit_horizontal(&mut self.viewport, &self.metrics),
            Command::FitVertical => zoom::fit_vertical(&mut self.viewport, &self.metrics),
            Command::ZoomStep => zoom::step(&mut self.viewport),
            Command::ZoomToPercent(percent) => {
                zoom::to_percent(&mut self.viewport, &self.metrics, percent)
            }
            Command::ToggleFrame => self.frame.enabled = !self.frame.enabled,
            Command::ShowMoreImages => {
                self.images_shown = (self.images_shown + 1).min(MAX_IMAGES_SHOWN);
            }
            Command::ShowFewerImages => self.images_shown = (self.images_shown - 1).max(1),
            Command::UserAction(index) => self.run_user_action(index, ctx),
        }
    }

    /// True while the current image is not ready and the user asked to wait
    /// rather than flick past unrendered images.
    fn should_wait(&self) -> bool {
        self.config.should_wait
            && matches!(
                self.store
                    .state((self.cursor + 1) % self.store.len().max(1)),
                ImageState::Loading
            )
    }

    fn run_user_action(&mut self, index: usize, ctx: &egui::Context) {
        let (Some(action), Some(path)) = (self.config.user_actions.get(index), self.active_path())
        else {
            return;
        };

        if !actions::execute(&action.exec, &path) {
            return;
        }

        if let Some(callback) = action.callback.clone() {
            self.callback = Some(Callback::from_callback(callback, Some(path)));
        }

        ctx.request_repaint();
    }

    /// Lays out the visible images side by side and draws each.
    fn show_images(&mut self, ctx: &egui::Context) -> Response {
        let background = self.background_colour();
        let shown = layout::show(
            ctx,
            &mut self.store,
            self.cursor,
            self.images_shown,
            &mut self.viewport,
            &self.frame,
            background,
        );

        // Zooming and the status bar both work from the geometry of the image
        // the user is looking at.
        if shown.metrics.image_size != Vec2::ZERO {
            self.metrics = shown.metrics;
        }

        shown.response
    }

    fn background_colour(&self) -> Color32 {
        self.slideshow
            .as_ref()
            .and(
                self.slideshow_config
                    .image_frame_background_color_override
                    .as_ref(),
            )
            .and_then(|hex| Color32::from_hex(hex).ok())
            .unwrap_or(BACKGROUND)
    }

    fn show_bottom_bar(&mut self, ctx: &egui::Context, flags: Flags) {
        let name = self.display_name();
        let total = self.store.len();
        let mut status = Status {
            jump_to: &mut self.jump_to,
            zoom: &mut self.viewport.zoom,
            // One based for the user, and zero when there is nothing open.
            position: total.min(self.cursor + 1),
            total,
            name,
            percentage_zoom: self.metrics.percentage_zoom,
            flags: Flags {
                filling: self.viewport.maximize,
                ..flags
            },
        };

        let outcome = bottom_bar::ui(ctx, &mut status);

        if let Some(index) = outcome.jump_to {
            self.select(index);
        }

        for command in outcome.commands {
            self.apply(command, ctx);
        }
    }

    /// The image's name, expanded through the configured metadata format.
    fn display_name(&self) -> String {
        let Some(metadata) = self.store.metadata(self.cursor) else {
            return self
                .store
                .path(self.cursor)
                .and_then(Path::file_name)
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
        };

        if self.config.name_format.is_empty() {
            return metadata.tags.get("File Name").cloned().unwrap_or_default();
        }

        crate::metadata::format_string_with_metadata(&self.config.name_format, &metadata.tags)
    }

    /// Scroll and drag over the image: navigation, zoom and panning.
    fn handle_pointer(&mut self, ctx: &egui::Context, response: &Response) {
        let hovered = response.contains_pointer();

        if self.config.scroll_navigation {
            if let Some(command) = input::scroll_navigation(ctx, hovered) {
                self.apply(command, ctx);
            }
        }

        let zoom_delta = ctx.input(|i| i.zoom_delta());
        if zoom_delta != 1.0 {
            zoom::by(&mut self.viewport, zoom_delta);
        }

        if !hovered {
            // Losing the pointer mid-scroll would otherwise leave the last
            // delta applied every frame.
            self.viewport.scroll_delta = Vec2::ZERO;
            return;
        }

        let mut delta = ctx.input(|i| i.smooth_scroll_delta);
        if ctx.input(|i| i.pointer.is_decidedly_dragging()) {
            delta += ctx.input(|i| i.pointer.delta()) * ctx.pixels_per_point();
        }

        self.viewport.scroll_delta = delta;
    }

    fn handle_context_menu(&mut self, response: &Response) {
        let Some(path) = self.active_path() else {
            return;
        };

        if let Some(callback) =
            actions::show_context_menu(&self.config.context_menu, response, &path)
        {
            self.callback = Some(Callback::from_callback(callback, Some(path)));
        }
    }

    fn run_slideshow(&mut self, ctx: &egui::Context) {
        let Some(slideshow) = &mut self.slideshow else {
            return;
        };

        let step = slideshow.tick();
        let zooms = slideshow.zooms();

        if step.advance {
            self.next_image();
        } else if zooms {
            // The base zoom fills the panel; the slideshow drifts in from there.
            let base = canvas::fill_zoom(self.metrics.fit_size, self.metrics.available_size);
            self.viewport.zoom = base * step.zoom_scale;
        }

        ctx.request_repaint_after(step.repaint_after);
    }
}

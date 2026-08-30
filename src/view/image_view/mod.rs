//! The main view: one (or a few) images filling the window.

pub mod bottom_bar;
pub mod canvas;
pub mod input;
pub mod interaction;
pub mod layout;
pub mod navigate;
pub mod slideshow;
pub mod viewports;
pub mod zoom;

use std::path::Path;
use std::sync::Arc;

use eframe::egui::{self, Response};
use eframe::egui_wgpu::RenderState;
use eframe::epaint::{Color32, Vec2};

use crate::actions::Callback;
use crate::cache::loader::Loader;
use crate::cache::{ImageState, ImageStore, StoreConfig};
use crate::config::{ImageViewConfig, Motion, SlideshowConfig};

use bottom_bar::{Flags, Marks, Status};
use canvas::{travelled, FrameStyle, Metrics, Style, Viewport};
use input::Command;
use layout::BACKGROUND;
use slideshow::Slideshow;
use viewports::{Place, Viewports};

/// Most images the view will place side by side. Beyond a handful they are too
/// small to read, and each one costs a texture.
const MAX_IMAGES_SHOWN: usize = 8;

pub struct ImageView {
    store: ImageStore,
    cursor: usize,
    viewport: Viewport,
    frame: FrameStyle,
    metrics: Metrics,
    /// Where the user got to in each image they zoomed, so coming back to one
    /// shows the same corner at the same magnification.
    viewports: Viewports,
    /// Where the image before this one was left, whether or not it was worth
    /// remembering. What "repeat the last view" repeats.
    previous_place: Place,
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
            viewports: Viewports::default(),
            previous_place: Place::UNTOUCHED,
            images_shown: config.nr_images_shown.clamp(1, MAX_IMAGES_SHOWN),
            jump_to: String::new(),
            callback: None,
            slideshow,
            slideshow_config,
            config,
        }
    }

    /// Tells the store how many pixels the screen can show, so decoders can
    /// stop at that size instead of producing a hundred megabytes nothing can
    /// display.
    pub fn set_display_edge(&mut self, edge: u32) {
        self.store.set_display_edge(edge);
    }

    /// Services the caches without drawing, so the view is ready the moment it
    /// is shown again.
    pub fn warm(&mut self) -> bool {
        self.store.tick()
    }

    /// Draws the view and services its caches.
    ///
    /// `marks` are the stars, flag and label on the image on screen, which the status
    /// bar shows so rating with the panel closed is not silent.
    pub fn ui(&mut self, ctx: &egui::Context, flags: Flags, marks: Marks) {
        if self.warm() {
            ctx.request_repaint();
        }

        for command in input::collect(ctx, &self.config) {
            self.apply(command, ctx);
        }

        if self.slideshow.is_none() {
            self.show_bottom_bar(ctx, flags, marks);
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
            Command::ZoomBy(factor) => zoom::by(&mut self.viewport, factor),
            Command::ZoomToPercent(percent) => {
                zoom::to_percent(&mut self.viewport, &self.metrics, percent)
            }
            Command::RepeatPlace => Viewports::put(&mut self.viewport, self.previous_place),
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
                // A thumbnail standing in is not the image being ready.
                ImageState::Loading | ImageState::Previewed
            )
    }

    fn show_images(&mut self, ctx: &egui::Context) -> Response {
        let background = self.background_colour();
        let shown = layout::show(
            ctx,
            &mut self.store,
            self.cursor,
            self.images_shown,
            &mut self.viewport,
            &Style {
                frame: self.frame,
                enlarge: self.config.enlarge_to_fit,
            },
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

    fn show_bottom_bar(&mut self, ctx: &egui::Context, flags: Flags, marks: Marks) {
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
            marks,
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
    fn run_slideshow(&mut self, ctx: &egui::Context) {
        let Some(slideshow) = &mut self.slideshow else {
            return;
        };

        let step = slideshow.tick();
        let motion = slideshow.motion();

        if step.advance {
            self.next_image();
        } else {
            self.animate(motion, step.zoom_scale, step.progress);
        }

        ctx.request_repaint_after(step.repaint_after);
    }

    /// Moves the viewport for this frame of the slideshow.
    fn animate(&mut self, motion: Motion, zoom_scale: f32, progress: f32) {
        // Filling the panel is the base every motion but the still one starts
        // from: the picture keeps its shape and covers the screen, cropping
        // whichever side is too long.
        let filling = canvas::fill_zoom(self.metrics.fit_size, self.metrics.available_size);

        match motion {
            Motion::Still => {}
            Motion::Zoom => self.viewport.zoom = filling * zoom_scale,
            Motion::Reveal => {
                self.viewport.zoom = filling;
                self.viewport.pan = travelled(
                    self.metrics.fit_size * filling,
                    self.metrics.available_size,
                    progress,
                );
            }
        }
    }
}

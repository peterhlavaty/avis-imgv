//! The main view: one (or a few) images filling the window.

pub mod bottom_bar;
pub mod canvas;
pub mod input;
pub mod interaction;
pub mod layout;
pub mod marks;
pub mod navigate;
pub mod overlay;
pub mod slideshow;
pub mod viewports;
pub mod zoom;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Response};
use eframe::egui_wgpu::RenderState;
use eframe::epaint::{Color32, Vec2};

use crate::actions::Callback;
use crate::cache::loader::Loader;
use crate::cache::{ImageState, ImageStore, StoreConfig};
use crate::config::{ImageViewConfig, Motion, SlideshowConfig};

use bottom_bar::{BarAction, Flags, Marks, Status};
use canvas::{travelled, FrameStyle, Metrics, Style, Viewport};

use crate::ui::empty::{Asked, Nothing};
use crate::ui::menus::Verb;
use crate::utils;
use crate::view::visible::Visible;
use input::Command;
use layout::BACKGROUND;
use slideshow::Slideshow;
use viewports::{Place, Viewports};

/// Most images the view will place side by side. Beyond a handful they are too
/// small to read, and each one costs a texture.
/// How many photographs a comparison starts with.
///
/// Two, because a comparison is nearly always between two frames of the same
/// thing; `Ctrl + Plus` widens it from there.
const COMPARE_PANES: usize = 2;

/// How far `Page Up` and `Page Down` move.
///
/// A round number rather than a screenful, because the image view shows one
/// photograph and a screenful of one is one.
const PAGE: usize = 10;

const MAX_IMAGES_SHOWN: usize = 8;

/// What a zoom holds still: the middle of the picture, or the point the
/// pointer is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Anchor {
    Centre,
    Pointer,
}

use Anchor::Centre as CENTRE;
use Anchor::Pointer as POINTER;

pub struct ImageView {
    store: ImageStore,
    /// Position in the store, not in what is on show.
    cursor: usize,
    /// Which of the store's photographs are being walked through.
    visible: Visible,
    /// What is being marked over the photograph, if anything.
    marking: crate::decoder::overlays::Overlay,
    /// The mask itself, held for the photograph on screen.
    marks: marks::Marks,
    /// A fixed set of photographs being compared against one another, rather
    /// than the run of neighbours the side-by-side view shows.
    ///
    /// The difference is what makes it a comparison: the panes stay where they
    /// are while the eye moves between them, and every key is about the one
    /// with the focus.
    comparing: Option<Vec<usize>>,
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
    /// A verb from the context menu that this view cannot carry out itself.
    verb: Option<(Verb, PathBuf)>,
    /// What the status bar was clicked to do, on its way to the application.
    bar_actions: Vec<BarAction>,
    /// What the screen with nothing on it was clicked to do.
    asked: Option<Asked>,
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
            visible: Visible::default(),
            marking: crate::decoder::overlays::Overlay::default(),
            marks: marks::Marks::default(),
            comparing: None,
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
            verb: None,
            bar_actions: Vec::new(),
            asked: None,
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
    pub fn ui(&mut self, ctx: &egui::Context, flags: Flags, marks: Marks, nothing: &Nothing) {
        if self.warm() {
            ctx.request_repaint();
        }

        for command in input::collect(ctx, &self.config) {
            self.apply(command, ctx);
        }

        if self.slideshow.is_none() {
            self.show_bottom_bar(ctx, flags, marks);
        }

        let response = self.show_images(ctx, nothing);
        self.handle_pointer(ctx, &response);
        self.handle_context_menu(ctx, &response);
        self.run_slideshow(ctx);
    }

    /// Applies a zoom and moves the pan so that a chosen point stays where it
    /// is.
    ///
    /// Zooming used to keep the middle of the *panel* whatever the user was
    /// looking at, so magnifying something near an edge pushed it further out
    /// of sight with every step — the one thing zoom is for.
    fn zooming(
        &mut self,
        ctx: &egui::Context,
        anchor: Anchor,
        change: impl FnOnce(&mut Viewport, &Metrics),
    ) {
        let held = match anchor {
            Anchor::Centre => Vec2::splat(0.5),
            Anchor::Pointer => self.pointer_anchor(ctx),
        };

        let before = self.viewport.zoom;
        change(&mut self.viewport, &self.metrics);

        self.viewport.pan = zoom::hold(
            &self.metrics,
            self.viewport.pan,
            before,
            self.viewport.zoom,
            held,
        );
    }

    /// Where the pointer is over the drawn photograph, nought to one.
    ///
    /// The middle when it is somewhere else entirely, because a keyboard zoom
    /// should not depend on where the mouse happens to be resting.
    fn pointer_anchor(&self, ctx: &egui::Context) -> Vec2 {
        let rect = self.metrics.rect;
        let Some(at) = ctx.input(|i| i.pointer.latest_pos()) else {
            return Vec2::splat(0.5);
        };

        if !rect.contains(at) || rect.width() <= 0.0 || rect.height() <= 0.0 {
            return Vec2::splat(0.5);
        }

        Vec2::new(
            (at.x - rect.left()) / rect.width(),
            (at.y - rect.top()) / rect.height(),
        )
    }

    fn apply(&mut self, command: Command, ctx: &egui::Context) {
        match command {
            Command::Next => {
                if !self.swap_focused_pane(true) {
                    self.next_image();
                }
            }
            Command::Previous => {
                if !self.swap_focused_pane(false) {
                    self.previous_image();
                }
            }
            Command::Compare => {
                if self.is_comparing() {
                    self.stop_comparing();
                } else {
                    self.start_comparing(COMPARE_PANES);
                }
            }
            Command::NextPane => {
                self.focus_next_pane();
                // Tab is also how egui walks its widgets, and the field it
                // lands in mutes every shortcut in the viewer.
                utils::surrender_focus(ctx);
            }
            Command::DropPane => self.drop_focused_pane(),
            Command::StopComparing => self.stop_comparing(),
            Command::First => self.jump_to_end(false),
            Command::Last => self.jump_to_end(true),
            Command::PageForward => self.page(true, PAGE),
            Command::PageBack => self.page(false, PAGE),
            // Fitting and filling are about the panel rather than about a
            // point in the picture, so they hold its middle; everything that
            // magnifies holds whatever is under the pointer.
            Command::Fit => self.zooming(ctx, CENTRE, |viewport, _| zoom::fit(viewport)),
            Command::Fill => self.zooming(ctx, CENTRE, zoom::fill),
            Command::ToggleFillLatch => {
                self.viewport.maximize = !self.viewport.maximize;
                self.viewport.maximized = false;
            }
            Command::FitHorizontal => self.zooming(ctx, CENTRE, zoom::fit_horizontal),
            Command::FitVertical => self.zooming(ctx, CENTRE, zoom::fit_vertical),
            Command::ZoomStep => {
                self.zooming(ctx, POINTER, |viewport, _| zoom::step(viewport));
            }
            Command::ZoomBy(factor) => {
                self.zooming(ctx, POINTER, |viewport, _| zoom::by(viewport, factor));
            }
            Command::ZoomToPercent(percent) => {
                self.zooming(ctx, POINTER, |viewport, metrics| {
                    zoom::to_percent(viewport, metrics, percent)
                });
            }
            Command::RepeatPlace => Viewports::put(&mut self.viewport, self.previous_place),
            Command::ToggleFrame => self.frame.enabled = !self.frame.enabled,
            Command::CycleOverlay => {
                self.config.overlay_corner = self.config.overlay_corner.next();
            }
            Command::CycleMarks => self.marking = self.marking.next(),
            Command::ShowMoreImages => {
                if !self.widen_comparison() {
                    self.images_shown = (self.images_shown + 1).min(MAX_IMAGES_SHOWN);
                }
            }
            Command::ShowFewerImages => {
                if !self.narrow_comparison() {
                    self.images_shown = (self.images_shown - 1).max(1);
                }
            }
            Command::UserAction(index) => self.run_user_action(index, ctx),
        }
    }

    /// True while the next image is not ready and the user asked to wait
    /// rather than flick past unrendered images.
    fn should_wait(&self) -> bool {
        if !self.config.should_wait {
            return false;
        }

        let (at, _) = self.position();
        let Some(next) = self
            .visible
            .next(at)
            .and_then(|position| self.visible.at(position))
        else {
            return false;
        };

        matches!(
            self.store.state(next),
            // A thumbnail standing in is not the image being ready.
            ImageState::Loading | ImageState::Previewed
        )
    }

    /// The store positions of the panes on screen, left to right.
    ///
    /// While comparing, the set that was pinned; otherwise the neighbours in
    /// what is on show rather than in the store, because with the rejects
    /// hidden the picture beside this one should be the next one a person
    /// would reach, not the one they have already said no to.
    /// Whether there is any photograph to draw.
    pub fn shows_nothing(&self) -> bool {
        self.panes().is_empty()
    }

    fn panes(&self) -> Vec<usize> {
        if let Some(comparing) = &self.comparing {
            return comparing.clone();
        }

        let (at, shown) = self.position();
        if shown == 0 {
            return Vec::new();
        }

        (0..self.images_shown.min(shown))
            .filter_map(|offset| self.visible.at((at + offset) % shown))
            .collect()
    }

    /// Whether a comparison is up.
    pub fn is_comparing(&self) -> bool {
        self.comparing.is_some()
    }

    /// Pins the photograph on screen and its neighbours as a comparison.
    ///
    /// The panes are what is on show around the cursor, so a filter narrows
    /// what can be compared the same way it narrows everything else.
    pub fn start_comparing(&mut self, count: usize) {
        let (at, shown) = self.position();
        if shown == 0 {
            return;
        }

        let panes: Vec<usize> = (0..count.clamp(2, MAX_IMAGES_SHOWN).min(shown))
            .filter_map(|offset| self.visible.at((at + offset) % shown))
            .collect();

        if panes.len() < 2 {
            return;
        }

        self.comparing = Some(panes);
    }

    /// Leaves the comparison, keeping the photograph the keys were about.
    pub fn stop_comparing(&mut self) {
        self.comparing = None;
    }

    /// Puts a different photograph in the focused pane.
    ///
    /// The motion Lightroom's compare view has: one pane stays as the one to
    /// beat and the arrow keys try the others against it, which is what
    /// choosing between a burst of near-identical frames actually is.
    pub fn swap_focused_pane(&mut self, forward: bool) -> bool {
        let Some(panes) = self.comparing.clone() else {
            return false;
        };

        let Some(at) = self.visible.position_of(self.cursor) else {
            return false;
        };

        let wanted = match forward {
            true => self.visible.next(at),
            false => self.visible.previous(at),
        }
        .and_then(|position| self.visible.at(position));

        let Some(index) = wanted else {
            return false;
        };

        // A photograph already in another pane would make two of the panes the
        // same picture, which compares nothing.
        if panes.contains(&index) {
            return true;
        }

        let Some(panes) = &mut self.comparing else {
            return false;
        };

        if let Some(slot) = panes.iter_mut().find(|pane| **pane == self.cursor) {
            *slot = index;
        }

        self.select(index);
        true
    }

    /// Adds the next photograph on show to the comparison.
    pub fn widen_comparison(&mut self) -> bool {
        let Some(panes) = self.comparing.clone() else {
            return false;
        };

        if panes.len() >= MAX_IMAGES_SHOWN {
            return true;
        }

        let (_, shown) = self.position();
        let last = panes.last().copied().unwrap_or(self.cursor);

        let wanted = self
            .visible
            .position_of(last)
            .and_then(|at| self.visible.next(at))
            .and_then(|position| self.visible.at(position));

        if let (Some(index), true) = (wanted, panes.len() < shown) {
            if let Some(panes) = &mut self.comparing {
                if !panes.contains(&index) {
                    panes.push(index);
                }
            }
        }

        true
    }

    /// Takes the last pane back off.
    pub fn narrow_comparison(&mut self) -> bool {
        let Some(panes) = &mut self.comparing else {
            return false;
        };

        if panes.len() <= 2 {
            return true;
        }

        let going = panes.pop();

        if going == Some(self.cursor) {
            if let Some(index) = self
                .comparing
                .as_ref()
                .and_then(|panes| panes.first())
                .copied()
            {
                self.select(index);
            }
        }

        true
    }

    /// Moves the focus to the next pane, wrapping round.
    pub fn focus_next_pane(&mut self) {
        let Some(panes) = self.comparing.clone() else {
            return;
        };

        let at = panes.iter().position(|index| *index == self.cursor);
        let next = match at {
            Some(at) => (at + 1) % panes.len(),
            None => 0,
        };

        if let Some(index) = panes.get(next) {
            self.select(*index);
        }
    }

    /// Drops the focused pane from the comparison, and the survivors re-tile.
    ///
    /// The elimination gesture: a comparison narrows to a winner rather than
    /// being decided in one go.
    pub fn drop_focused_pane(&mut self) {
        let Some(panes) = &mut self.comparing else {
            return;
        };

        if panes.len() <= 2 {
            // Two is the fewest a comparison can be; dropping one of them ends
            // it on the other, which is the answer.
            let survivor = panes.iter().find(|index| **index != self.cursor).copied();
            self.comparing = None;

            if let Some(index) = survivor {
                self.select(index);
            }

            return;
        }

        let going = self.cursor;
        panes.retain(|index| *index != going);

        if let Some(index) = self.comparing.as_ref().and_then(|panes| panes.first()) {
            let index = *index;
            self.select(index);
        }
    }

    fn show_images(&mut self, ctx: &egui::Context, nothing: &Nothing) -> Response {
        self.prepare_marks(ctx);

        let background = self.background_colour();
        let panes = self.panes();

        // Worked out before the store is handed over, because it reads the
        // very store the drawing borrows.
        let style = Style {
            overlay: self.overlay(),
            // Over the picture, through its own texture coordinates, so the
            // mask follows the zoom and the pan without knowing about either.
            // Only the pane the keys are on: a comparison marks the frame
            // being judged rather than all of them.
            mask: self.marks.texture_id(),
            frame: self.frame,
            enlarge: self.config.enlarge_to_fit,
        };

        let shown = layout::show(
            ctx,
            &mut self.store,
            &panes,
            self.cursor,
            &mut self.viewport,
            &layout::Painting {
                style: &style,
                background,
                nothing,
            },
        );

        self.asked = shown.asked;

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
        let (at, total) = self.position();
        let hidden = self.store.len() - total;
        let comparing = self.is_comparing();
        let mut status = Status {
            jump_to: &mut self.jump_to,
            // One based for the user, and zero when there is nothing open.
            position: total.min(at + 1),
            total,
            hidden,
            name,
            percentage_zoom: self.metrics.percentage_zoom,
            marks,
            flags: Flags {
                filling: self.viewport.maximize,
                comparing,
                marking: self.marking,
                ..flags
            },
        };

        let outcome = bottom_bar::ui(ctx, &mut status);

        if let Some(position) = outcome.jump_to {
            self.select_position(position);
        }

        for command in outcome.commands {
            self.apply(command, ctx);
        }

        self.bar_actions.extend(outcome.bar);
    }

    /// What the screen with nothing on it was clicked to do.
    pub fn take_asked(&mut self) -> Option<Asked> {
        self.asked.take()
    }

    /// What the status bar was clicked to do that the view cannot do itself.
    pub fn take_bar_actions(&mut self) -> Vec<BarAction> {
        std::mem::take(&mut self.bar_actions)
    }

    /// Builds the clipping or focus mask for the photograph on screen.
    ///
    /// From the decoded copy that is already in RAM, which is the same pixels
    /// the texture holds — so the mask lines up exactly and no file is read
    /// again. Nothing is built while the overlay is off.
    fn prepare_marks(&mut self, ctx: &egui::Context) {
        if self.marking == crate::decoder::overlays::Overlay::Off {
            self.marks.forget();
            return;
        }

        let (Some(path), Some(decoded)) = (
            self.store.path(self.cursor).map(Path::to_path_buf),
            self.store.decoded(self.cursor),
        ) else {
            return;
        };

        let surface = &decoded.surface;
        let (pixels, width, height) = (surface.pixels.clone(), surface.width, surface.height);

        self.marks
            .prepare(ctx, self.marking, &path, &pixels, width, height);
    }

    /// What to write over the photograph, already expanded.
    ///
    /// Once per frame rather than once per pane: every pane would render the
    /// same template, and the answer is about the photograph the keys are on.
    fn overlay(&self) -> canvas::Overlay {
        let corner = self.config.overlay_corner;
        if corner == overlay::Corner::Off || self.config.overlay_format.is_empty() {
            return canvas::Overlay::default();
        }

        let Some(path) = self.store.path(self.cursor) else {
            return canvas::Overlay::default();
        };

        let mut subject = crate::metadata::template::Subject::new(path);
        if let Some(metadata) = self.store.metadata(self.cursor) {
            subject = subject.with_metadata(metadata);
        }

        canvas::Overlay {
            corner,
            lines: crate::metadata::template::render(&self.config.overlay_format, &subject),
            size: self.config.overlay_text_size,
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

        // The same grammar the rename and the captions use, so a status bar
        // line can say anything a file name can and the other way round.
        let Some(path) = self.store.path(self.cursor) else {
            return String::new();
        };

        let subject = crate::metadata::template::Subject::new(path).with_metadata(metadata);

        crate::metadata::template::render(&self.config.name_format, &subject)
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

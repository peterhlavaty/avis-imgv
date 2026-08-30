//! The contact sheet: every image in the folder as a thumbnail.
//!
//! Thumbnails come from a store of their own so the grid can hold hundreds of
//! small textures without competing for the budget the full size view needs.

pub mod cell;
pub mod layout;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, scroll_area::ScrollSource, Color32, Rect, Sense, UiBuilder};
use eframe::egui_wgpu::RenderState;
use eframe::epaint::Vec2;

use crate::actions::{self, Callback};
use crate::cache::loader::Loader;
use crate::cache::{ImageState, ImageStore, StoreConfig, StoreStats};
use crate::config::{shortcut, GridViewConfig};
use crate::utils;
use crate::view::texture;

use crate::view::image_view::bottom_bar::Marks;
use crate::view::visible::Visible;

use cell::Badges;
use layout::Layout;

const CELL_BACKGROUND: Color32 = Color32::from_rgb(119, 119, 119);
const CELL_BORDER: Color32 = Color32::from_rgb(48, 48, 48);

/// A thumbnail is never cropped, so it always shows all of itself.
const WHOLE_IMAGE: Rect = Rect {
    min: eframe::epaint::pos2(0.0, 0.0),
    max: eframe::epaint::pos2(1.0, 1.0),
};

/// Widest the grid will go before more images stop fitting usefully.
const MAX_COLUMNS: usize = 16;

pub struct GridView {
    store: ImageStore,
    config: GridViewConfig,
    columns: usize,
    /// Set when the user picks an image, consumed by the app.
    selected: Option<PathBuf>,
    callback: Option<Callback>,
    /// Image to scroll to on the next frame.
    scroll_to: Option<usize>,
    /// Where the keyboard is, as a position in what is on show. Not where the
    /// image view is: moving about a contact sheet should not decode a full
    /// sized photograph at every step.
    cursor: usize,
    /// Which photograph the image view is on, as a store position, so the
    /// sheet can say so.
    current: usize,
    badges: Badges,
    /// Which of the store's photographs are shown, and in what order.
    visible: Visible,
}

impl GridView {
    pub fn new(
        render_state: RenderState,
        loader: Arc<Loader>,
        store_config: StoreConfig,
        output_profile: Arc<str>,
        config: GridViewConfig,
    ) -> GridView {
        GridView {
            store: ImageStore::new(render_state, loader, store_config, output_profile),
            columns: config.images_per_row.max(1),
            config,
            selected: None,
            callback: None,
            scroll_to: None,
            cursor: 0,
            current: 0,
            badges: Badges::default(),
            visible: Visible::default(),
        }
    }

    pub fn set_images(&mut self, paths: Vec<PathBuf>) {
        self.visible = Visible::everything(paths.len());
        self.store.set_paths(paths);
        self.scroll_to = Some(0);
        self.cursor = 0;
        self.current = 0;
    }

    /// Narrows or reorders the sheet, keeping the cursor where it can.
    pub fn set_visible(&mut self, visible: Visible) {
        let staying = self.visible.at(self.cursor);
        self.visible = visible;

        self.cursor = staying
            .and_then(|index| self.visible.nearest(index))
            .unwrap_or(0);
        self.scroll_to = Some(self.cursor);
    }

    pub fn stats(&self) -> StoreStats {
        self.store.stats()
    }

    /// The image the user picked, if any. Consumed on read.
    pub fn take_selected(&mut self) -> Option<PathBuf> {
        self.selected.take()
    }

    pub fn take_callback(&mut self) -> Option<Callback> {
        self.callback.take()
    }

    pub fn pop(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.store.remove(index);
            self.visible.remove_shifting(index);
            self.cursor = self.cursor.min(self.visible.len().saturating_sub(1));
        }
    }

    pub fn reload(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.store.reload(index);
        }
    }

    /// Services the caches without drawing, so opening the grid does not start
    /// from nothing.
    pub fn warm(&mut self, cursor: usize) -> bool {
        self.current = cursor;
        self.store.set_cursor(cursor);
        self.store.tick()
    }

    /// Takes a changed configuration, for when the keyboard map is edited.
    pub fn set_config(&mut self, config: GridViewConfig) {
        self.config = config;
    }

    /// Scrolls to `index` on the next frame drawn.
    ///
    /// Asked for when the gallery is opened, and only then: doing it every
    /// frame would drag the view back to the open image the instant the user
    /// scrolled away from it.
    pub fn focus_on(&mut self, index: usize) {
        self.current = index;
        self.cursor = self.visible.nearest(index).unwrap_or(0);
        self.scroll_to = Some(self.cursor);
    }

    /// The store position the keyboard is on, so the panels follow the sheet.
    pub fn cursor(&self) -> Option<usize> {
        self.visible.at(self.cursor)
    }

    pub fn cursor_path(&self) -> Option<PathBuf> {
        self.cursor()
            .and_then(|index| self.store.path(index))
            .map(Path::to_path_buf)
    }

    /// Draws the grid.
    ///
    /// `marks` is what every photograph in the collection carries, in the same
    /// order, so the sheet can draw them without asking the disk per cell.
    pub fn ui(&mut self, ctx: &egui::Context, marks: &[Marks]) {
        if self.store.tick() {
            ctx.request_repaint();
        }

        self.handle_input(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let shown = self.visible.len();
            let layout = Layout::new(
                ui.available_width(),
                self.columns,
                shown,
                self.config.cell_aspect,
                self.badges.caption_height(),
            );

            if shown == 0 {
                let says = if self.store.is_empty() {
                    "No images here"
                } else {
                    "Nothing matches the filter"
                };

                ui.centered_and_justified(|ui| ui.label(says));
                return;
            }

            let mut scroll_area = egui::ScrollArea::vertical().scroll_source(ScrollSource::ALL);
            if let Some(index) = self.scroll_to.take() {
                scroll_area = scroll_area.vertical_scroll_offset(layout.scroll_offset_of(index));
            }

            scroll_area.show_rows(ui, layout.row, layout.rows, |ui, rows| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;

                // Caching centres on what is on screen, so scrolling pulls the
                // rows just past the fold in ahead of the user. The middle of
                // the fold is a position in what is shown; the store wants the
                // photograph that position stands for.
                let onscreen = layout.indices(rows.clone(), shown);
                let middle = (onscreen.start + onscreen.end) / 2;

                if let Some(index) = self.visible.at(middle) {
                    self.store.set_cursor(index);
                }

                for row in rows {
                    self.show_row(ui, &layout, row, marks);
                }

                if !utils::are_inputs_muted(ctx)
                    && ui.input_mut(|i| shortcut::consume(i, &self.config.sc_scroll))
                {
                    ui.scroll_with_delta(Vec2::new(0., -(layout.row * 0.5)));
                }
            });
        });
    }

    fn show_row(&mut self, ui: &mut egui::Ui, layout: &Layout, row: usize, marks: &[Marks]) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.add_space(layout.padding);

            for position in layout.indices(row..row + 1, self.visible.len()) {
                let Some(index) = self.visible.at(position) else {
                    continue;
                };

                self.show_cell(ui, position, index, layout, marks.get(index));
            }
        });
    }

    fn show_cell(
        &mut self,
        ui: &mut egui::Ui,
        position: usize,
        index: usize,
        layout: &Layout,
        marks: Option<&Marks>,
    ) {
        let (_, rect) = ui.allocate_space(Vec2::new(layout.cell, layout.row));
        let picture = Rect::from_min_size(rect.min, Vec2::new(layout.cell, layout.picture));
        let strip = Rect::from_min_max(
            eframe::epaint::pos2(rect.left(), picture.bottom()),
            rect.max,
        );

        ui.painter().rect_filled(picture, 0, CELL_BACKGROUND);

        let name = self.file_name(index);
        let drawn = self.store.texture(index).is_some();

        let response = ui
            .scope_builder(UiBuilder::new().max_rect(picture), |ui| {
                ui.centered_and_justified(|ui| {
                    if !drawn {
                        show_placeholder(ui, self.store.state(index), layout.picture);
                        return None;
                    }

                    // Borrowed again inside, because the placeholder branch
                    // needs the store and the drawing branch needs the
                    // texture.
                    let texture = self.store.texture(index)?;
                    let size = fit_in_cell(texture.size, layout.cell, layout.picture);
                    let (drawn_rect, response) = ui.allocate_exact_size(size, Sense::click());
                    texture::draw(ui, drawn_rect, texture, WHOLE_IMAGE);

                    Some(response.on_hover_text_at_pointer(&name))
                })
                .inner
            })
            .inner;

        cell::dim_if_rejected(ui, picture, marks);
        cell::caption(ui, strip, self.badges, marks, &name);

        ui.painter().rect_stroke(
            rect,
            0.,
            egui::Stroke::new(1.0_f32, CELL_BORDER),
            egui::StrokeKind::Outside,
        );

        cell::outline(ui, rect, index == self.current, position == self.cursor);

        if let Some(response) = response {
            self.handle_cell_interaction(ui, position, index, &response);
        }
    }

    fn handle_cell_interaction(
        &mut self,
        ui: &egui::Ui,
        position: usize,
        index: usize,
        response: &egui::Response,
    ) {
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let Some(path) = self.store.path(index).map(Path::to_path_buf) else {
            return;
        };

        if response.clicked() {
            self.cursor = position;
            self.selected = Some(path.clone());
        }

        if let Some(callback) =
            actions::show_context_menu(&self.config.context_menu, response, &path)
        {
            self.callback = Some(Callback::from_callback(callback, Some(path)));
        }
    }

    /// Which photograph the sheet says is on show, and how many there are.
    pub fn position(&self) -> (usize, usize) {
        (self.cursor, self.visible.len())
    }

    fn file_name(&self, index: usize) -> String {
        self.store
            .path(index)
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        if utils::are_inputs_muted(ctx) {
            return;
        }

        let zooming = ctx.input(|i| i.zoom_delta() != 1.0);
        let scroll = ctx.input(|i| i.raw_scroll_delta.y);

        let wider = ctx.input_mut(|i| shortcut::consume(i, &self.config.sc_more_per_row))
            || (zooming && scroll < 0.);
        let narrower = ctx.input_mut(|i| shortcut::consume(i, &self.config.sc_less_per_row))
            || (zooming && scroll > 0.);

        if wider && self.columns < MAX_COLUMNS {
            self.set_columns(self.columns + 1);
        } else if narrower && self.columns > 1 {
            self.set_columns(self.columns - 1);
        }

        if ctx.input_mut(|i| shortcut::consume(i, &self.config.sc_cycle_badges)) {
            self.badges = self.badges.next();
        }

        self.move_cursor(ctx);
    }

    /// Walks the sheet with the arrow keys, and opens with Enter.
    ///
    /// A contact sheet nobody can move about without a mouse is a contact
    /// sheet nobody can cull from: every mark is a keystroke, and reaching the
    /// next photograph should be one too.
    fn move_cursor(&mut self, ctx: &egui::Context) {
        let total = self.visible.len();
        if total == 0 {
            return;
        }

        let columns = self.columns.max(1);
        let steps = [
            (egui::Key::ArrowRight, 1_isize),
            (egui::Key::ArrowLeft, -1),
            (egui::Key::ArrowDown, columns as isize),
            (egui::Key::ArrowUp, -(columns as isize)),
        ];

        let mut moved = false;

        for (key, step) in steps {
            if !ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                continue;
            }

            let wanted = self.cursor as isize + step;
            // Clamped rather than wrapped: a sheet has edges, and walking off
            // one to land at the far end of another row is disorienting.
            self.cursor = wanted.clamp(0, total as isize - 1) as usize;
            moved = true;
        }

        for (key, index) in [
            (egui::Key::Home, 0usize),
            (egui::Key::End, total.saturating_sub(1)),
        ] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                self.cursor = index;
                moved = true;
            }
        }

        if moved {
            self.scroll_to_cursor();
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            self.selected = self.cursor_path();
        }
    }

    /// Brings the cursor into view, without dragging the sheet about when it
    /// is already there.
    fn scroll_to_cursor(&mut self) {
        self.scroll_to = Some(self.cursor);
    }

    /// Changes the column count, keeping the user roughly where they were.
    fn set_columns(&mut self, columns: usize) {
        self.scroll_to = Some(self.cursor);
        self.columns = columns;
    }
}

/// Largest size with the thumbnail's aspect ratio that fits the cell.
fn fit_in_cell(size: Vec2, width: f32, height: f32) -> Vec2 {
    if size.x <= 0.0 || size.y <= 0.0 {
        return Vec2::new(width, height);
    }

    let scale = (width / size.x).min(height / size.y);
    size * scale
}

fn show_placeholder(ui: &mut egui::Ui, state: ImageState, cell: f32) {
    if state == ImageState::Failed {
        ui.label("✖");
        return;
    }

    ui.add(egui::Spinner::new().size(cell / 3.));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnails_keep_their_aspect_ratio_in_a_square_cell() {
        assert_eq!(
            fit_in_cell(Vec2::new(200.0, 100.0), 100.0, 100.0),
            Vec2::new(100.0, 50.0)
        );
        assert_eq!(
            fit_in_cell(Vec2::new(100.0, 200.0), 100.0, 100.0),
            Vec2::new(50.0, 100.0)
        );
        assert_eq!(
            fit_in_cell(Vec2::new(100.0, 100.0), 100.0, 100.0),
            Vec2::new(100.0, 100.0)
        );
    }

    /// The cell a three-to-two photograph is drawn into is now three to two
    /// itself, so that photograph fills it and a portrait one is letterboxed.
    #[test]
    fn a_matching_photograph_fills_the_cell() {
        assert_eq!(
            fit_in_cell(Vec2::new(6000.0, 4000.0), 300.0, 200.0),
            Vec2::new(300.0, 200.0)
        );

        let portrait = fit_in_cell(Vec2::new(4000.0, 6000.0), 300.0, 200.0);
        assert!((portrait.y - 200.0).abs() < 0.01, "{portrait:?}");
        assert!((portrait.x - 400.0 / 3.0).abs() < 0.01, "{portrait:?}");
    }

    #[test]
    fn a_degenerate_thumbnail_fills_the_cell() {
        assert_eq!(fit_in_cell(Vec2::ZERO, 64.0, 48.0), Vec2::new(64.0, 48.0));
    }
}

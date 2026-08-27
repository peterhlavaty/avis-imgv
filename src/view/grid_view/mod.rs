//! The contact sheet: every image in the folder as a thumbnail.
//!
//! Thumbnails come from a store of their own so the grid can hold hundreds of
//! small textures without competing for the budget the full size view needs.

pub mod layout;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, scroll_area::ScrollSource, Color32, Rect, Sense, UiBuilder};
use eframe::egui_wgpu::RenderState;
use eframe::epaint::Vec2;

use crate::actions::{self, Callback};
use crate::cache::loader::Loader;
use crate::cache::{ImageState, ImageStore, StoreConfig, StoreStats};
use crate::config::GridViewConfig;
use crate::utils;
use crate::view::texture;

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
        }
    }

    pub fn set_images(&mut self, paths: Vec<PathBuf>) {
        self.store.set_paths(paths);
        self.scroll_to = Some(0);
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
        self.scroll_to = Some(index);
    }

    /// Draws the grid.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if self.store.tick() {
            ctx.request_repaint();
        }

        self.handle_input(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let layout = Layout::new(ui.available_width(), self.columns, self.store.len());

            let mut scroll_area = egui::ScrollArea::vertical().scroll_source(ScrollSource::ALL);
            if let Some(index) = self.scroll_to.take() {
                scroll_area = scroll_area.vertical_scroll_offset(layout.scroll_offset_of(index));
            }

            scroll_area.show_rows(ui, layout.cell, layout.rows, |ui, rows| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;

                // Caching centres on what is on screen, so scrolling pulls the
                // rows just past the fold in ahead of the user.
                let visible = layout.indices(rows.clone(), self.store.len());
                self.store.set_cursor((visible.start + visible.end) / 2);

                for row in rows {
                    self.show_row(ui, &layout, row);
                }

                if !utils::are_inputs_muted(ctx)
                    && ui.input_mut(|i| i.consume_shortcut(&self.config.sc_scroll.kbd_shortcut))
                {
                    ui.scroll_with_delta(Vec2::new(0., -(layout.cell * 0.5)));
                }
            });
        });
    }

    fn show_row(&mut self, ui: &mut egui::Ui, layout: &Layout, row: usize) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.add_space(layout.padding);

            for index in layout.indices(row..row + 1, self.store.len()) {
                self.show_cell(ui, index, layout.cell);
            }
        });
    }

    fn show_cell(&mut self, ui: &mut egui::Ui, index: usize, cell: f32) {
        let (_, rect) = ui.allocate_space(Vec2::splat(cell));
        ui.painter().rect_filled(rect, 0, CELL_BACKGROUND);

        let name = self.file_name(index);
        let drawn = self.store.texture(index).is_some();

        let response = ui
            .scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.centered_and_justified(|ui| {
                    if !drawn {
                        show_placeholder(ui, self.store.state(index), cell);
                        return None;
                    }

                    // Borrowed again inside, because the placeholder branch
                    // needs the store and the drawing branch needs the
                    // texture.
                    let texture = self.store.texture(index)?;
                    let size = fit_in_cell(texture.size, cell);
                    let (drawn_rect, response) = ui.allocate_exact_size(size, Sense::click());
                    texture::draw(ui, drawn_rect, texture, WHOLE_IMAGE);

                    Some(response.on_hover_text_at_pointer(&name))
                })
                .inner
            })
            .inner;

        ui.painter().rect_stroke(
            rect,
            0.,
            egui::Stroke::new(1.0_f32, CELL_BORDER),
            egui::StrokeKind::Outside,
        );

        if let Some(response) = response {
            self.handle_cell_interaction(ui, index, &response);
        }
    }

    fn handle_cell_interaction(&mut self, ui: &egui::Ui, index: usize, response: &egui::Response) {
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let Some(path) = self.store.path(index).map(Path::to_path_buf) else {
            return;
        };

        if response.clicked() {
            self.selected = Some(path.clone());
        }

        if let Some(callback) =
            actions::show_context_menu(&self.config.context_menu, response, &path)
        {
            self.callback = Some(Callback::from_callback(callback, Some(path)));
        }
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

        let wider = ctx
            .input_mut(|i| i.consume_shortcut(&self.config.sc_more_per_row.kbd_shortcut))
            || (zooming && scroll < 0.);
        let narrower = ctx
            .input_mut(|i| i.consume_shortcut(&self.config.sc_less_per_row.kbd_shortcut))
            || (zooming && scroll > 0.);

        if wider && self.columns < MAX_COLUMNS {
            self.set_columns(self.columns + 1);
        } else if narrower && self.columns > 1 {
            self.set_columns(self.columns - 1);
        }
    }

    /// Changes the column count, keeping the user roughly where they were.
    fn set_columns(&mut self, columns: usize) {
        self.scroll_to = Some(self.store.cursor());
        self.columns = columns;
    }
}

/// Largest size with the thumbnail's aspect ratio that fits a square cell.
fn fit_in_cell(size: Vec2, cell: f32) -> Vec2 {
    if size.x <= 0.0 || size.y <= 0.0 {
        return Vec2::splat(cell);
    }

    let aspect = size.x / size.y;
    if aspect > 1.0 {
        Vec2::new(cell, cell / aspect)
    } else {
        Vec2::new(cell * aspect, cell)
    }
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
            fit_in_cell(Vec2::new(200.0, 100.0), 100.0),
            Vec2::new(100.0, 50.0)
        );
        assert_eq!(
            fit_in_cell(Vec2::new(100.0, 200.0), 100.0),
            Vec2::new(50.0, 100.0)
        );
        assert_eq!(
            fit_in_cell(Vec2::new(100.0, 100.0), 100.0),
            Vec2::new(100.0, 100.0)
        );
    }

    #[test]
    fn a_degenerate_thumbnail_fills_the_cell() {
        assert_eq!(fit_in_cell(Vec2::ZERO, 64.0), Vec2::splat(64.0));
    }
}

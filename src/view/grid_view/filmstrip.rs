//! A strip of thumbnails under the photograph.
//!
//! The contact sheet answers "what is in this folder" and the image view
//! answers "what is this frame", and between them there is a question neither
//! does well: what is either side of the one I am looking at. Culling is a
//! walk along a line of frames, and a viewer that shows one frame at a time
//! makes that walk blind — you cannot see the burst you are in the middle of
//! without leaving the picture.
//!
//! From the thumbnail store the contact sheet already fills, so it costs
//! nothing but the drawing: the textures are resident whichever view is on
//! screen, which is why the grid is warmed while the image view is up.
//!
//! It follows what is on show rather than the whole folder, so a filtered
//! collection has a filtered strip — the frames it skips past are the frames
//! it does not draw.

use eframe::egui::{self, Color32, Rect, Sense};
use eframe::epaint::Vec2;

use crate::cache::{ImageState, ImageStore};
use crate::view::texture;
use crate::view::visible::Visible;

/// The thumbnail is never cropped, so it always shows all of itself.
const WHOLE_IMAGE: Rect = Rect {
    min: eframe::epaint::pos2(0.0, 0.0),
    max: eframe::epaint::pos2(1.0, 1.0),
};

const BACKGROUND: Color32 = Color32::from_rgb(38, 38, 38);
const CELL: Color32 = Color32::from_rgb(70, 70, 70);

/// The border round the photograph the viewer is on.
const CURRENT: Color32 = Color32::from_rgb(232, 232, 232);

/// Gap either side of a cell.
const GAP: f32 = 3.0;

/// What the strip reports back.
pub struct Picked {
    /// A store position the user clicked on.
    pub selected: Option<usize>,
    /// The strip was dragged to this height.
    ///
    /// Through the field the settings window reads, so a dragged edge survives
    /// the session — which is the thing none of this program's in-view controls
    /// used to do.
    pub height: Option<f32>,
}

/// Draws the strip, `height` points tall.
///
/// `cursor` is the store position on screen, so the strip can mark it and
/// scroll to keep it in view.
pub fn show(
    ctx: &egui::Context,
    store: &mut ImageStore,
    visible: &Visible,
    cursor: usize,
    height: f32,
) -> Picked {
    let mut picked = Picked {
        selected: None,
        height: None,
    };

    if height <= 0.0 || visible.is_empty() {
        return picked;
    }

    let panel = egui::TopBottomPanel::bottom("filmstrip")
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(BACKGROUND).inner_margin(4.0))
        .resizable(true)
        .default_height(height)
        .min_height(48.0)
        .max_height(400.0)
        .show(ctx, |ui| {
            ui.interact(
                ui.max_rect(),
                ui.id().with("strip hover"),
                egui::Sense::hover(),
            )
            .on_hover_text(
                "Every photograph in the folder, in order. Click one to open it; \n                 drag the top edge to make the strip taller.",
            );

            ui.spacing_mut().item_spacing = Vec2::new(GAP, 0.0);

            let cell = (height - 8.0).max(16.0);
            let at = visible.position_of(cursor);

            let mut area =
                egui::ScrollArea::horizontal().scroll_source(egui::scroll_area::ScrollSource::ALL);

            // Kept in view as the cursor moves, which is the whole reason to
            // have it: a strip showing where you were is no use.
            if let Some(at) = at {
                let step = cell + GAP;
                let middle = ui.available_width() / 2.0;
                area = area.horizontal_scroll_offset((at as f32 * step - middle).max(0.0));
            }

            area.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(GAP, 0.0);

                    for index in visible.iter() {
                        let name = store
                            .path(index)
                            .and_then(|path| path.file_name())
                            .map(|name| name.to_string_lossy().into_owned());

                        if draw_cell(ui, store, index, index == cursor, cell, name.as_deref()) {
                            picked.selected = Some(index);
                        }
                    }
                });
            });
        });

    // The dragged height, reported so it reaches the configuration.
    let dragged = panel.response.rect.height();
    if (dragged - height).abs() > 1.0 {
        picked.height = Some(dragged);
    }

    picked
}

/// One thumbnail. Returns whether it was clicked.
fn draw_cell(
    ui: &mut egui::Ui,
    store: &mut ImageStore,
    index: usize,
    current: bool,
    cell: f32,
    name: Option<&str>,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(cell), Sense::click());

    // Nothing is drawn for a cell that has scrolled past: a strip over a folder
    // of ten thousand would otherwise ask the store about every one of them
    // every frame.
    if !ui.is_rect_visible(rect) {
        return false;
    }

    ui.painter().rect_filled(rect, 0.0, CELL);

    match store.texture(index) {
        Some(texture) => {
            let size = fit(texture.size, cell);
            let at = Rect::from_center_size(rect.center(), size);

            texture::draw(ui, at, texture, WHOLE_IMAGE);
        }
        None => {
            // A cell the decoders have not reached says so quietly rather than
            // spinning: a strip of forty spinners is a strobe.
            if store.state(index) == ImageState::Failed {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✖",
                    egui::FontId::proportional(cell * 0.3),
                    Color32::from_rgb(150, 150, 150),
                );
            }
        }
    }

    if current {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            0.0,
            egui::Stroke::new(2.0_f32, CURRENT),
            egui::StrokeKind::Inside,
        );
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Named, because a strip of forty thumbnails forty pixels across is a
    // strip in which nothing can be read.
    if let Some(name) = name {
        response.clone().on_hover_text(name);
    }

    response.clicked()
}

/// Largest size with the thumbnail's shape that fits a square cell.
fn fit(size: Vec2, cell: f32) -> Vec2 {
    if size.x <= 0.0 || size.y <= 0.0 {
        return Vec2::splat(cell);
    }

    size * (cell / size.x).min(cell / size.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compared with a tolerance: the scale is a division and a multiply, so
    /// a square cell can come back a few bits off sixty.
    fn close(found: Vec2, wanted: Vec2) -> bool {
        (found.x - wanted.x).abs() < 0.01 && (found.y - wanted.y).abs() < 0.01
    }

    #[test]
    fn a_thumbnail_keeps_its_shape_in_a_square_cell() {
        for (size, cell, wanted) in [
            (Vec2::new(200.0, 100.0), 100.0, Vec2::new(100.0, 50.0)),
            (Vec2::new(100.0, 200.0), 100.0, Vec2::new(50.0, 100.0)),
            (Vec2::new(100.0, 100.0), 60.0, Vec2::new(60.0, 60.0)),
            (Vec2::new(6000.0, 4000.0), 48.0, Vec2::new(48.0, 32.0)),
        ] {
            let found = fit(size, cell);
            assert!(close(found, wanted), "{size:?} in {cell}: {found:?}");
        }
    }

    #[test]
    fn a_thumbnail_of_no_size_fills_the_cell() {
        assert_eq!(fit(Vec2::ZERO, 48.0), Vec2::splat(48.0));
    }
}

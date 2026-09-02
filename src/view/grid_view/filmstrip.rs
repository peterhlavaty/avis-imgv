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

/// The margin inside the panel, on each of the four sides.
const MARGIN: f32 = 4.0;

/// Room kept under the thumbnails for the horizontal scroll bar.
///
/// Reserved rather than overlaid: a bar drawn across the bottom of a row of
/// thumbnails hides the part of a photograph that a cull is often about.
const BAR: f32 = 14.0;

/// The shortest and the tallest the strip may be dragged to.
///
/// The top matches the range the registry row declares, so the number typed
/// into the settings window and the number the edge can reach are the same
/// number.
pub const SHORTEST: f32 = 48.0;
pub const TALLEST: f32 = 400.0;

/// The largest square cell a strip that many points tall can draw.
///
/// Split out and free of egui so the arithmetic that decides how big the
/// thumbnails are can be tested without a window. `inner` is what the panel
/// leaves after its own margins, which is what the strip actually has.
pub fn cell_side(inner: f32) -> f32 {
    (inner - BAR).max(16.0)
}

/// What the strip reports back.
pub struct Picked {
    /// A store position the user clicked on.
    pub selected: Option<usize>,
    /// How tall the panel is actually drawn, this frame.
    ///
    /// Reported every frame rather than only when it differs, because telling
    /// a drag from a layout pass is [`crate::ui::dragged::Dragged`]'s job and
    /// it needs to see the frames in between to do it.
    pub height: f32,
}

/// Draws the strip, `height` points tall.
///
/// `cursor` is the store position on screen, so the strip can mark it and
/// scroll to keep it in view. `forced` states the height rather than
/// suggesting it, for the one frame after something other than a drag has
/// changed it — the settings window, or the history putting it back.
///
/// The contents are made to fill the panel, and that is not decoration. egui
/// remembers a panel's size as the rectangle its *contents* came to, not the
/// rectangle the drag asked for, so a strip whose thumbnails were sized from
/// the configured height reported that height back however far the edge had
/// been pulled — and the panel returned to where it started on the very next
/// frame. Filling the height makes the two rectangles the same one.
pub fn show(
    ctx: &egui::Context,
    store: &mut ImageStore,
    visible: &Visible,
    cursor: usize,
    height: f32,
    forced: bool,
) -> Picked {
    let mut picked = Picked {
        selected: None,
        height,
    };

    if height <= 0.0 || visible.is_empty() {
        return picked;
    }

    let mut panel = egui::TopBottomPanel::bottom("filmstrip")
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(BACKGROUND).inner_margin(MARGIN))
        .resizable(true)
        .default_height(height)
        .min_height(SHORTEST)
        .max_height(TALLEST);

    if forced {
        panel = panel.exact_height(height);
    }

    let panel = panel.show(ctx, |ui| {
            // Whatever the drag has just asked for, rather than what the
            // configuration last said: the cells follow the edge in the same
            // frame it moves, and the panel keeps the size it was given.
            let inner = ui.available_height();
            ui.set_min_height(inner);

            ui.interact(
                ui.max_rect(),
                ui.id().with("strip hover"),
                egui::Sense::hover(),
            )
            .on_hover_text(
                "Every photograph in the folder, in order. Click one to open it; \n                 drag the top edge to make the strip taller.",
            );

            ui.spacing_mut().item_spacing = Vec2::new(GAP, 0.0);

            let cell = cell_side(inner);
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

    picked.height = panel.response.rect.height();

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

    if crate::utils::is_a_window_in_front(ui.ctx()) {
        return false;
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

    /// The point of dragging the strip taller: the thumbnails grow with it.
    /// They did not, because the cell was sized from the configured height
    /// rather than from the height the panel had actually been given.
    #[test]
    fn a_taller_strip_draws_larger_thumbnails() {
        let mut last = 0.0_f32;

        for inner in [SHORTEST, 96.0, 200.0, TALLEST] {
            let cell = cell_side(inner);
            assert!(cell > last, "{inner} gave {cell}, no larger than {last}");
            last = cell;
        }
    }

    /// The scroll bar gets room of its own rather than being drawn across the
    /// bottom of the photographs.
    #[test]
    fn the_scroll_bar_is_not_drawn_over_a_thumbnail() {
        assert!(cell_side(200.0) <= 200.0 - BAR);
    }

    /// The shortest the edge can be dragged to still draws something worth
    /// looking at rather than collapsing to the floor.
    #[test]
    fn the_shortest_strip_still_draws_a_thumbnail() {
        assert!(cell_side(SHORTEST) >= 16.0);
    }

    /// A height below the floor is still answered with a cell rather than a
    /// negative one: a hand-edited configuration reaches here.
    #[test]
    fn a_strip_of_no_height_still_answers_with_a_cell() {
        assert_eq!(cell_side(0.0), 16.0);
        assert_eq!(cell_side(-40.0), 16.0);
    }
}

//! Placing the visible images in the central panel.

use eframe::egui::{self, Response, Sense};
use eframe::epaint::{Color32, Vec2};

use crate::cache::{ImageState, ImageStore};

use super::canvas::{self, Metrics, Style, Viewport};

/// Backdrop behind the images, neutral enough not to shift how a photograph
/// reads against it.
pub const BACKGROUND: Color32 = Color32::from_rgb(119, 119, 119);

/// What one frame of the central panel produced.
pub struct Shown {
    /// The panel itself, for pointer handling and the context menu.
    pub response: Response,
    /// Geometry of the image under the cursor.
    pub metrics: Metrics,
}

/// Draws `count` images starting at `cursor`, side by side.
///
/// Only the first is measured and only the first is uploaded on demand: it is
/// the one the user is looking at, and the one the zoom commands act on.
pub fn show(
    ctx: &egui::Context,
    store: &mut ImageStore,
    cursor: usize,
    count: usize,
    viewport: &mut Viewport,
    style: &Style,
    background: Color32,
) -> Shown {
    let mut metrics = Metrics::default();
    let total = store.len();

    let response = egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(background))
        .show(ctx, |ui| {
            if total == 0 {
                ui.centered_and_justified(|ui| ui.label("No images here"));
                return;
            }

            let count = count.clamp(1, total);
            let cell = Vec2::new(
                (ui.available_width() / count as f32) - 1.,
                ui.available_height(),
            );

            ui.horizontal(|ui| {
                for offset in 0..count {
                    let index = (cursor + offset) % total;
                    ui.allocate_ui(cell, |ui| {
                        ui.centered_and_justified(|ui| {
                            let drawn = show_one(ui, store, index, offset == 0, viewport, style);
                            if offset == 0 {
                                if let Some(drawn) = drawn {
                                    metrics = drawn;
                                }
                            }
                        });
                    });
                }
            });
        })
        .response
        .interact(Sense::click());

    Shown { response, metrics }
}

fn show_one(
    ui: &mut egui::Ui,
    store: &mut ImageStore,
    index: usize,
    urgent: bool,
    viewport: &mut Viewport,
    style: &Style,
) -> Option<Metrics> {
    // The image under the cursor jumps the per-frame upload budget; the ones
    // beside it can wait a frame.
    let texture = if urgent {
        store.texture_now(index)
    } else {
        store.texture(index)
    };

    let Some(texture) = texture else {
        placeholder(ui, store.state(index));
        return None;
    };

    let metrics = canvas::draw(ui, texture, viewport, style);

    // How wide it ended up being drawn decides which copy of it should be on
    // the GPU: the screen sized one while it fits, the image's own pixels once
    // the user magnifies past that.
    store.set_drawn_width(index, metrics.drawn_width);

    Some(metrics)
}

/// Shows why an image is not on screen yet.
fn placeholder(ui: &mut egui::Ui, state: ImageState) {
    if state == ImageState::Failed {
        ui.label("Could not open this image");
        return;
    }

    let size = ui.available_height() / 3.;
    ui.add(egui::Spinner::new().size(size));
}

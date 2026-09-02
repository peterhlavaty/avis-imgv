//! Placing the visible images in the central panel.

use eframe::egui::{self, Response, Sense};
use eframe::epaint::{Color32, Vec2};

use crate::cache::{ImageState, ImageStore};
use crate::ui::empty::{self, Asked, Nothing};

use super::canvas::{self, Metrics, Style, Viewport};
use super::comparison::{self, Banner};

/// Backdrop behind the images, neutral enough not to shift how a photograph
/// reads against it.
///
/// The default of `general.backdrop`, which is what is actually drawn.
pub const BACKGROUND: Color32 = Color32::from_rgb(119, 119, 119);

/// What one frame of the central panel produced.
pub struct Shown {
    /// The panel itself, for pointer handling and the context menu.
    pub response: Response,
    /// Geometry of the image under the cursor.
    pub metrics: Metrics,
    /// What the screen with nothing on it was clicked to do.
    pub asked: Option<Asked>,
    /// Where each pane was drawn, so a press can be told which photograph it
    /// landed on and the controls over each of them have somewhere to go.
    ///
    /// Computed here already — it is what the focus outline is drawn round —
    /// and thrown away until now, which is why the menu over a comparison of
    /// four was about the focused photograph whichever of the four the button
    /// came down on.
    pub panes: Vec<(usize, egui::Rect)>,
}

/// How the panel is painted, as against what is in it.
pub struct Painting<'a> {
    pub style: &'a Style,
    /// The grey behind the photograph.
    pub background: Color32,
    /// What to draw when there is no photograph to draw.
    pub nothing: &'a Nothing,
    /// What a pinned comparison says it is about, when one is up.
    pub comparison: Option<Banner>,
    /// What that comparison is outlined and named in.
    pub comparison_colour: Color32,
}

/// Draws `count` images starting at `cursor`, side by side.
///
/// Only the first is measured and only the first is uploaded on demand: it is
/// the one the user is looking at, and the one the zoom commands act on.
pub fn show(
    ctx: &egui::Context,
    store: &mut ImageStore,
    panes: &[usize],
    focused: Option<usize>,
    viewport: &mut Viewport,
    painting: &Painting<'_>,
) -> Shown {
    let Painting {
        style,
        background,
        nothing,
        comparison,
        comparison_colour,
    } = painting;
    let background = *background;
    let comparison_colour = *comparison_colour;

    let mut metrics = Metrics::default();
    let mut asked = None;
    let mut drawn: Vec<(usize, egui::Rect)> = Vec::new();

    let response = egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(background))
        .show(ctx, |ui| {
            if panes.is_empty() {
                asked = empty::ui(ui, nothing);
                return;
            }

            let cell = Vec2::new(
                (ui.available_width() / panes.len() as f32) - 1.,
                ui.available_height(),
            );

            let several = panes.len() > 1;

            ui.horizontal(|ui| {
                for index in panes {
                    let leading = Some(*index) == focused;

                    let allocated = ui.allocate_ui(cell, |ui| {
                        ui.centered_and_justified(|ui| {
                            let drawn = show_one(ui, store, *index, leading, viewport, style);
                            if leading {
                                if let Some(drawn) = drawn {
                                    metrics = drawn;
                                }
                            }
                        });
                    });

                    drawn.push((*index, allocated.response.rect));

                    // Which pane the keys are about has to be unmistakable, or
                    // marking one of four is a guess. One pane needs no
                    // marking, because there is nothing to tell it apart from.
                    if several && leading {
                        ui.painter().rect_stroke(
                            allocated.response.rect.shrink(1.0),
                            0.0,
                            egui::Stroke::new(2.0_f32, FOCUSED),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            });

            // Last, over the photographs: the outline is about the panel
            // rather than about anything in it. The name and the cross are
            // drawn by the caller, in a layer of their own — see `comparison`.
            if comparison.is_some() {
                comparison::outline(ui, ui.max_rect(), comparison_colour);
            }
        })
        .response
        .interact(Sense::click());

    Shown {
        response,
        metrics,
        asked,
        panes: drawn,
    }
}

/// `leading` is the pane the keys are about: it owns the viewport, it is
/// measured, and it jumps the per-frame upload budget.
fn show_one(
    ui: &mut egui::Ui,
    store: &mut ImageStore,
    index: usize,
    leading: bool,
    viewport: &mut Viewport,
    style: &Style,
) -> Option<Metrics> {
    // The image being looked at jumps the per-frame upload budget; the ones
    // beside it can wait a frame.
    let texture = if leading {
        store.texture_now(index)
    } else {
        store.texture(index)
    };

    let Some(texture) = texture else {
        placeholder(ui, store.state(index));
        return None;
    };

    let metrics = canvas::draw(ui, texture, viewport, style, leading);

    // How wide it ended up being drawn decides which copy of it should be on
    // the GPU: the screen sized one while it fits, the image's own pixels once
    // the user magnifies past that.
    store.set_drawn_width(index, metrics.drawn_width);

    Some(metrics)
}

/// The border round the pane the keys are about.
const FOCUSED: Color32 = Color32::from_rgb(126, 168, 224);

/// Shows why an image is not on screen yet.
fn placeholder(ui: &mut egui::Ui, state: ImageState) {
    if state == ImageState::Failed {
        ui.label("Could not open this image");
        return;
    }

    let size = ui.available_height() / 3.;
    ui.add(egui::Spinner::new().size(size));
}

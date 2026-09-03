//! Drawing the tones of a photograph.
//!
//! Three curves and a brightness fill, the way every photographic tool draws
//! them, plus the two numbers a screen cannot show: what proportion of the
//! frame has clipped at each end. A monitor renders 250 and 255 as the same
//! white, so "is that sky recoverable" is a question the picture itself cannot
//! answer and this can.

use eframe::egui::{self, Color32, RichText, Sense, Stroke};
use eframe::epaint::{pos2, Rect, Vec2};

use crate::decoder::histogram::{Histogram, BUCKETS};
use crate::decoder::overlays::Overlay;

/// How tall the plot is, in points.
const HEIGHT: f32 = 90.0;

/// The three channels, drawn additively so grey shows where they agree.
const RED: Color32 = Color32::from_rgba_premultiplied(120, 30, 30, 0);
const GREEN: Color32 = Color32::from_rgba_premultiplied(30, 120, 30, 0);
const BLUE: Color32 = Color32::from_rgba_premultiplied(30, 30, 130, 0);
const LUMA: Color32 = Color32::from_rgba_premultiplied(70, 70, 70, 0);

/// The colour a clipping figure is called out in, once there is enough of it
/// to matter.
const WARNING: Color32 = Color32::from_rgb(219, 160, 96);

/// Below this a percentage is a rounding error rather than a problem: a
/// handful of specular highlights on water clip in every photograph ever
/// taken, and calling that out would train people to ignore the number.
const WORTH_SAYING: f32 = 0.1;

/// What a figure was clicked to do.
///
/// `Blown 3.4%` and `Crushed 0.2%` were true statements nobody could act on,
/// while the mask that marks exactly those pixels was a key in a different
/// subsystem. They are the same question asked twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// Paint the clipping mask over the photograph, or take it off again.
    Clipping,
    /// Open the keys for the mask, so it can be reached from where it is read.
    BindKey(&'static str),
}

/// Draws `histogram`, and the clipping figures under it.
///
/// `marking` is whichever mask is on the photograph, which the figures need in
/// order to offer the thing that is not already there.
pub fn show(ui: &mut egui::Ui, histogram: &Histogram, marking: Overlay) -> Option<Asked> {
    if histogram.is_empty() {
        return None;
    }

    ui.add_space(20.0);
    ui.label(RichText::new("Tones").heading())
        .on_hover_text(
            "How the photograph's tones are distributed, counted on the worker while \n             the pixels were already in hand. Left is black, right is white.",
        );
    ui.add_space(10.0);

    let width = ui.available_width().max(64.0);
    let (rect, area) = ui.allocate_exact_size(Vec2::new(width, HEIGHT), Sense::hover());
    area.on_hover_text(
        "Grey is brightness, which is the shape most people read; red, green and blue \n         are the channels behind it. A wall against either edge is clipping.",
    );
    let painter = ui.painter();

    painter.rect_filled(rect, 2.0, Color32::from_rgb(28, 28, 28));

    let tallest = histogram.tallest().max(1) as f32;

    // Brightness behind the three channels: it is the shape people read for
    // exposure, and the channels are what they read for a cast.
    for (channel, colour) in [
        (&histogram.luma, LUMA),
        (&histogram.red, RED),
        (&histogram.green, GREEN),
        (&histogram.blue, BLUE),
    ] {
        plot(painter, rect, channel, tallest, colour);
    }

    // The ends, so it is obvious which side is which without a label.
    painter.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 60)),
        egui::StrokeKind::Inside,
    );

    clipping(ui, histogram, marking)
}

/// What the figure offers, in the words for the state it finds.
///
/// The same sentence the mask's own word in the status bar carries, because it
/// is the same verb. Focus peaking is a mask this figure knows nothing about:
/// with it on, what a clipping figure offers is still to mark the clipping,
/// which replaces it.
fn verb(marking: Overlay) -> &'static str {
    match marking == Overlay::Clipping {
        true => "Show the photograph as it is",
        false => "Mark the clipping on the photograph",
    }
}

/// One channel, as a filled area.
fn plot(
    painter: &egui::Painter,
    rect: Rect,
    counts: &[u32; BUCKETS],
    tallest: f32,
    colour: Color32,
) {
    let step = rect.width() / BUCKETS as f32;

    // A column per bucket rather than a polyline: at two hundred points wide
    // there are fewer pixels than buckets, so a line would be a lie about
    // which values are present.
    for (bucket, count) in counts.iter().enumerate() {
        if *count == 0 {
            continue;
        }

        // The square root, which is what every photographic histogram uses:
        // a linear scale on a photograph with a large flat sky is one spike
        // and nothing else.
        let height = (*count as f32 / tallest).sqrt() * rect.height();
        let x = rect.left() + bucket as f32 * step;

        painter.rect_filled(
            Rect::from_min_max(
                pos2(x, rect.bottom() - height),
                pos2(x + step.max(1.0), rect.bottom()),
            ),
            0.0,
            colour,
        );
    }
}

/// What the picture cannot show: how much of it has gone at each end.
///
/// Each figure is the button for the mask that marks exactly the pixels it is
/// counting, which is what a person means when they read "Blown 3.4 %" and
/// looks for somewhere to click. The left button does it, the way the glyphs
/// in the status bar do; the menu carries the same verb written out, for
/// somebody who has not worked out that a number is a button, and the keys
/// behind it.
///
/// One mask and not two halves of one: it paints the blown pixels red and the
/// crushed ones blue in the same pass, and a photographer looking at either
/// end wants both marked. So both figures offer the same verb, and the heading
/// of the menu says which of them was asked.
fn clipping(ui: &mut egui::Ui, histogram: &Histogram, marking: Overlay) -> Option<Asked> {
    let mut asked = None;

    let verb = verb(marking);

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        for (label, percent, hover) in [
            (
                "Blown",
                histogram.blown_percent(),
                "Pixels with a channel at the top of the range, which hold no detail to recover",
            ),
            (
                "Crushed",
                histogram.crushed_percent(),
                "Pixels black in every channel",
            ),
        ] {
            let text = RichText::new(format!("{label} {percent:.1}%"));

            let figure = ui.add(
                egui::Label::new(if percent >= WORTH_SAYING {
                    text.color(WARNING)
                } else {
                    text.weak()
                })
                .sense(Sense::click()),
            );

            // The left button is the whole of what these are for: a number
            // that says a twentieth of the frame has gone is read while
            // deciding whether to keep the photograph, and the mask is the
            // answer. It sensed clicks and nothing read them.
            if figure.clicked() {
                asked = Some(Asked::Clipping);
            }

            let reading = format!("{percent:.1}%");

            crate::ui::surface::with_menu(
                ui,
                &figure,
                crate::ui::surface::Subject::of(label, &reading),
                &format!("{hover}. {verb}."),
                |ui| {
                    if crate::ui::keys::button(ui, verb, "image_view.sc_marks").clicked() {
                        asked = Some(Asked::Clipping);
                        ui.close();
                    }

                    // Where the menu ends, since the mask is a key and a
                    // runtime state and not a setting anywhere — the same last
                    // row the mask's own word in the status bar carries.
                    if crate::ui::surface::bind_a_key(ui, "the mask") {
                        asked = Some(Asked::BindKey("image_view.sc_marks"));
                        ui.close();
                    }
                },
            );
        }
    });

    asked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn histogram(pixels: &[[u8; 4]]) -> Histogram {
        let bytes: Vec<u8> = pixels.iter().flatten().copied().collect();

        Histogram::of(&bytes)
    }

    /// The threshold exists so that the number means something when it is
    /// called out: a few specular highlights clip in every photograph.
    #[test]
    fn a_trace_of_clipping_is_not_called_out() {
        let mut pixels = vec![[128, 128, 128, 255]; 2000];
        pixels[0] = [255, 255, 255, 255];

        let found = histogram(&pixels);

        assert!(
            found.blown_percent() < WORTH_SAYING,
            "{}",
            found.blown_percent()
        );
    }

    /// And a sky that has gone is.
    #[test]
    fn a_blown_sky_is_called_out() {
        let mut pixels = vec![[128, 128, 128, 255]; 100];
        for pixel in pixels.iter_mut().take(20) {
            *pixel = [255, 255, 255, 255];
        }

        let found = histogram(&pixels);

        assert!(found.blown_percent() >= WORTH_SAYING);
        assert_eq!(found.blown_percent(), 20.0);
    }

    /// Nothing decoded yet draws nothing rather than an empty box, which
    /// would flicker in and out while a folder loads.
    #[test]
    fn nothing_to_draw_is_nothing() {
        assert!(Histogram::default().is_empty());
    }

    /// The row offers what is not already there. It offered both — two rows,
    /// worded differently, that did the very same thing.
    #[test]
    fn the_row_offers_the_state_it_is_not_in() {
        assert_eq!(verb(Overlay::Off), "Mark the clipping on the photograph");
        assert_eq!(verb(Overlay::Clipping), "Show the photograph as it is");

        // Focus peaking is a mask this figure is not about, so what it offers
        // is still the clipping — which replaces it.
        assert_eq!(
            verb(Overlay::Peaking),
            "Mark the clipping on the photograph"
        );
    }

    /// The left button is the whole of what a figure is for, and nothing read
    /// it: the label sensed clicks and the response was passed to the menu and
    /// dropped.
    #[test]
    fn clicking_a_figure_asks_for_the_mask() {
        let blown = histogram(&[[255, 255, 255, 255]; 100]);
        let ctx = egui::Context::default();

        let mut asked = None;
        let draw = |ctx: &egui::Context, asked: &mut Option<Asked>| {
            egui::CentralPanel::default().show(ctx, |ui| {
                *asked = show(ui, &blown, Overlay::Off);
            });
        };

        // Where the figure landed, read off the frame that drew it, rather
        // than a position guessed from the spacing.
        let output = ctx.run(egui::RawInput::default(), |ctx| draw(ctx, &mut asked));
        let at = crate::ui::drawn::text_at(&output, "Blown 100.0%").expect("the figure is drawn");

        let press = |pressed: bool| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        };

        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::PointerMoved(at), press(true), press(false)],
                ..Default::default()
            },
            |ctx| draw(ctx, &mut asked),
        );

        assert_eq!(asked, Some(Asked::Clipping));
    }
}

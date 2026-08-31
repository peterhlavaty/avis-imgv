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

/// Draws `histogram`, and the clipping figures under it.
pub fn show(ui: &mut egui::Ui, histogram: &Histogram) {
    if histogram.is_empty() {
        return;
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

    clipping(ui, histogram);
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
fn clipping(ui: &mut egui::Ui, histogram: &Histogram) {
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

            ui.label(if percent >= WORTH_SAYING {
                text.color(WARNING)
            } else {
                text.weak()
            })
            .on_hover_text(hover);
        }
    });
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
}

//! What a test reads back off a frame that has been drawn.
//!
//! A widget in this program is often the thing a test is about — whether a row
//! says the key beside it, whether a menu opened, whether a figure answered the
//! button — and egui hands a test the shapes it painted and nothing else. So a
//! test draws a frame with `Context::run`, finds the text it is asking about
//! among those shapes, and where necessary aims a click at where it landed
//! rather than at a position guessed from the spacing.
//!
//! Here rather than at the foot of each of the modules that wants it: three of
//! them did, and two already held a copy of the same six lines.

use eframe::egui;

/// Every piece of text the frame painted, in the order it was painted.
pub fn text(output: &egui::FullOutput) -> Vec<String> {
    output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text) => Some(text.galley.text().to_string()),
            _ => None,
        })
        .collect()
}

/// Where a piece of text was painted, for a click aimed at it.
///
/// A little way inside its top-left corner, which is a point the widget that
/// drew it certainly covers.
pub fn text_at(output: &egui::FullOutput, wanted: &str) -> Option<egui::Pos2> {
    output
        .shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text) if text.galley.text() == wanted => {
                Some(text.pos + egui::vec2(4.0, 4.0))
            }
            _ => None,
        })
}

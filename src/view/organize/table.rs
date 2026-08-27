//! The list of what would happen, which is the whole point of both modes.
//!
//! Nothing is written until it has been looked at, so the table is not a
//! decoration: it is the thing the user is being asked to approve.

use eframe::egui::{self, Color32, RichText};

/// How many rows are drawn before the rest are summarised.
///
/// A folder can hold ten thousand files and every row is a laid out grid cell;
/// the first few hundred are enough to see whether a template is right.
const MAX_ROWS: usize = 500;

/// One line of the preview.
pub struct Row {
    pub before: String,
    pub after: String,
    /// Why this one will not happen, if it will not.
    pub problem: Option<String>,
    /// Whether this row would actually change anything.
    pub changes: bool,
}

/// Draws the preview, with `headings` naming the two columns.
pub fn show(ui: &mut egui::Ui, headings: (&str, &str), rows: &[Row]) {
    if rows.is_empty() {
        ui.add_space(12.0);
        ui.weak("No files match the filter.");
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("organize preview")
                .num_columns(3)
                .striped(true)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new(headings.0).strong());
                    ui.label(RichText::new(headings.1).strong());
                    ui.label("");
                    ui.end_row();

                    for row in rows.iter().take(MAX_ROWS) {
                        line(ui, row);
                    }
                });

            if rows.len() > MAX_ROWS {
                ui.add_space(6.0);
                ui.weak(format!(
                    "…and {} more, which will be included too",
                    rows.len() - MAX_ROWS
                ));
            }
        });
}

fn line(ui: &mut egui::Ui, row: &Row) {
    ui.label(&row.before);

    let after = RichText::new(&row.after);
    let after = match (&row.problem, row.changes) {
        (Some(_), _) => after.color(Color32::from_rgb(220, 120, 120)),
        // Nothing to do is worth showing as nothing to do rather than as a
        // change that happens to be identical.
        (None, false) => after.weak(),
        (None, true) => after,
    };

    ui.label(after);

    match &row.problem {
        Some(problem) => ui.weak(problem),
        None if !row.changes => ui.weak("unchanged"),
        None => ui.label(""),
    };

    ui.end_row();
}

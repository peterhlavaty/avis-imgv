//! The band across the top of the settings window.
//!
//! What `Config::check()` found, one row per complaint, each with a button that
//! opens the owning page and puts the cursor on the control. The route from the
//! complaint to the control is the whole value: every one of these failures used
//! to end in a log file whose path was written only into that same log.
//!
//! And a permanent red bar when a section of the file could not be read. A
//! section that fails to parse blocks all saving for the session; the user was
//! told once, for six seconds, and the section's name reached only the log.

use eframe::egui::{self, RichText};

use crate::config::Config;

use super::State;

/// What the band was asked for.
pub enum Asked {
    /// Open the page that owns this field and put the cursor on the control.
    Fix(&'static str),
    /// Stop showing the complaints for this session.
    Dismiss,
}

/// Draws the band, if there is anything to say.
pub fn band(ui: &mut egui::Ui, state: &State, config: &Config) -> Option<Asked> {
    let mut asked = None;

    if config.partial {
        partial(ui);
    }

    if state.problems.is_empty() {
        return asked;
    }

    egui::Frame::group(ui.style())
        .fill(WARNING_FILL)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} thing{} in the configuration file",
                        state.problems.len(),
                        if state.problems.len() == 1 { "" } else { "s" }
                    ))
                    .strong(),
                );

                if ui
                    .small_button("Not now")
                    .on_hover_text("Hides these until the viewer next starts")
                    .clicked()
                {
                    asked = Some(Asked::Dismiss);
                }
            });

            for complaint in &state.problems {
                ui.horizontal_wrapped(|ui| {
                    ui.label(&complaint.says);
                    ui.weak(&complaint.instead);

                    if !complaint.path.is_empty() && ui.small_button("Fix").clicked() {
                        asked = Some(Asked::Fix(complaint.path));
                    }
                });
            }
        });

    ui.add_space(8.0);
    asked
}

/// The permanent bar for a file that was only partly read.
///
/// Permanent because the consequence is: nothing can be saved for the rest of
/// the session, so every control below this is drawn disabled rather than
/// hidden — Microsoft's rule for an inapplicable page, whose reason is that
/// greying the whole thing out would force people to look on all the others.
fn partial(ui: &mut egui::Ui) {
    egui::Frame::group(ui.style())
        .fill(FAILURE_FILL)
        .show(ui, |ui| {
            ui.label(
                RichText::new("Part of the configuration file could not be read")
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            ui.label(
                "Those settings are at their defaults, and nothing can be written back \
                 until it is fixed — writing would replace whatever could not be read.",
            );

            ui.horizontal(|ui| {
                if ui.button("Show me the file").clicked() {
                    if let Some(path) = Config::path() {
                        crate::actions::reveal::in_file_manager(&path);
                    }
                }

                if ui
                    .button("Open the log")
                    .on_hover_text("The log names the section that could not be read, and why")
                    .clicked()
                {
                    if let Some(path) = crate::logging::path() {
                        crate::actions::reveal::with_the_system(&path);
                    }
                }
            });
        });

    ui.add_space(8.0);
}

const WARNING_FILL: egui::Color32 = egui::Color32::from_rgb(74, 58, 26);
const FAILURE_FILL: egui::Color32 = egui::Color32::from_rgb(72, 32, 32);

#[cfg(test)]
mod tests {
    use crate::config::Config;

    /// The band is empty for a file with nothing wrong with it, which is what
    /// makes it worth reading when it is not.
    #[test]
    fn a_good_file_has_an_empty_band() {
        assert!(Config::default().check().is_empty());
    }
}

//! What to do when the configuration file has been edited under the viewer.
//!
//! The file is read once at startup and the viewer holds a copy for the rest
//! of the run, so an in-app save writes over whatever was hand-edited
//! meanwhile. `Config::save` refuses in that case, and this is the question
//! that gets asked instead: read the file again, or keep what is on screen.

use eframe::egui;

/// What the person answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// Still up; nothing decided.
    Waiting,
    /// Throw away the in-memory copy and read the file.
    Reread,
    /// Write over the file with what the viewer holds.
    Overwrite,
}

/// Draws the question, and says what was answered.
pub fn contents(ui: &mut egui::Ui) -> Answer {
    let mut answer = Answer::Waiting;

    {
        ui.label(
            "Something has edited the file since the viewer read it, so the change \
                 just made was not written — writing it would have thrown that edit away.",
        );

        if let Some(path) = crate::config::Config::path() {
            ui.add_space(6.0);
            ui.weak(path.display().to_string());
        }

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui
                .button("Read the file again")
                .on_hover_text("Takes what is on disk and loses the change just made")
                .clicked()
            {
                answer = Answer::Reread;
            }

            if ui
                .button("Keep what is on screen")
                .on_hover_text("Writes over the file, losing whatever was edited there")
                .clicked()
            {
                answer = Answer::Overwrite;
            }
        });
    }

    answer
}

//! What this build is, and where it keeps its files.
//!
//! Three of the most confusing behaviours in the program are diagnosable from
//! this one card: which graphics adapter is being drawn on, whether this
//! build can develop a raw file at all, and where the configuration and the log
//! actually live — all three of which were decided at startup and told only to
//! the log, whose own path was written only into that same log.

use eframe::egui;

use crate::actions::reveal;

/// What the card says about the build, read once when it is constructed.
#[derive(Debug, Clone)]
pub struct About {
    pub version: &'static str,
    /// The graphics adapter, as wgpu named it.
    pub adapter: String,
    /// Whether this build can develop a raw file rather than only showing the
    /// preview the camera embedded in it.
    pub libraw: Option<String>,
}

impl About {
    /// One line saying what raw files do here.
    pub fn raw_line(&self) -> String {
        match &self.libraw {
            Some(version) => format!("Raw development available, LibRaw {version}"),
            None => "Built without LibRaw; raw files show their embedded preview".to_string(),
        }
    }
}

/// Draws the card. Nothing in it changes anything.
pub fn contents(ui: &mut egui::Ui, about: &About) {
    ui.heading("avis-imgv");
    ui.label(format!("Version {}", about.version));
    ui.add_space(8.0);

    ui.label(&about.adapter)
        .on_hover_text("The graphics adapter the photographs are drawn on");
    ui.label(about.raw_line())
        .on_hover_text("Decided when the program starts, from whether LibRaw could be found");

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(6.0);

    file_row(
        ui,
        "Configuration",
        crate::config::Config::path(),
        "Every setting, as JSON. The viewer reads it once at startup.",
    );
    file_row(
        ui,
        "Log",
        crate::logging::path(),
        "What the viewer has been doing, and what went wrong.",
    );
}

/// One path, with a copy button and a way to open it.
fn file_row(ui: &mut egui::Ui, name: &str, path: Option<std::path::PathBuf>, hint: &str) {
    ui.horizontal(|ui| {
        ui.label(format!("{name}:"));

        let Some(path) = path else {
            ui.weak("no configuration directory on this system");
            return;
        };

        let text = path.display().to_string();
        // Selectable, so it can be picked up by hand as well as by the button.
        ui.add(egui::Label::new(egui::RichText::new(&text).monospace()).wrap())
            .on_hover_text(hint);

        if ui
            .button("Copy")
            .on_hover_text("The path, on the clipboard")
            .clicked()
        {
            ui.ctx().copy_text(text);
        }

        if ui
            .button("Open")
            .on_hover_text("With whatever the system uses for it")
            .clicked()
        {
            reveal::with_the_system(&path);
        }

        if ui
            .button("Show me the folder")
            .on_hover_text("Opens the file manager with it picked out")
            .clicked()
        {
            reveal::in_file_manager(&path);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one sentence that answers "why is my raw so small".
    #[test]
    fn the_raw_line_says_which_build_this_is() {
        let with = About {
            version: "0.4.0",
            adapter: "test".into(),
            libraw: Some("0.21".into()),
        };
        assert!(with.raw_line().contains("0.21"));

        let without = About {
            libraw: None,
            ..with
        };
        assert!(without.raw_line().contains("embedded preview"));
    }
}

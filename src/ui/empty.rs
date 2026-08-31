//! What the viewer draws when it has nothing to draw.
//!
//! Four words on grey — "No images here" — was the first thing most people
//! saw, because with no argument the crawler reads the working directory,
//! which the source itself calls nobody's choice. It offered no way to pick a
//! folder, said nothing about the keys, and never mentioned that the session
//! file has been keeping a list of the last folders visited all along.

use std::path::PathBuf;

use eframe::egui;

/// How many recent folders the home screen offers.
///
/// Six: enough to cover the shoots somebody moves between, few enough that the
/// screen is a short list rather than a history to read.
pub const OFFERED: usize = 6;

/// What the empty screen was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    OpenFolder,
    OpenFiles,
    /// One of the folders visited lately.
    Open(PathBuf),
    /// Set the narrowing rules aside without forgetting them.
    ShowEverything,
}

/// The screen with no photographs on it.
///
/// Owned rather than borrowed, and built only on the frames where it is drawn:
/// the sentences are allocated, and an allocation per frame is an allocation
/// per photograph per frame.
#[derive(Debug, Default, Clone)]
pub struct Nothing {
    /// Whether there are photographs that the filter is holding back, as
    /// against no photographs at all.
    pub filtered: bool,
    /// What the rules currently say, for naming them.
    pub rules: Vec<String>,
    /// The folders visited lately, most recent first.
    pub recent: Vec<PathBuf>,
    /// Whether to name the keys, which is worth doing on a first run.
    pub say_the_keys: bool,
}

/// Draws it into whatever `ui` is available, and reports what was clicked.
pub fn ui(ui: &mut egui::Ui, nothing: &Nothing) -> Option<Asked> {
    let mut asked = None;

    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.22);

        if nothing.filtered {
            filtered(ui, nothing, &mut asked);
        } else {
            no_folder(ui, nothing, &mut asked);
        }
    });

    asked
}

/// There are photographs; the rules are hiding all of them.
fn filtered(ui: &mut egui::Ui, nothing: &Nothing, asked: &mut Option<Asked>) {
    ui.heading("Nothing matches the filter");
    ui.add_space(8.0);

    if nothing.rules.is_empty() {
        ui.weak("Every photograph in this folder is being held back.");
    } else {
        for rule in &nothing.rules {
            ui.weak(rule);
        }
    }

    ui.add_space(12.0);

    if ui
        .button("Show everything")
        .on_hover_text("Sets the rules aside without forgetting them")
        .clicked()
    {
        *asked = Some(Asked::ShowEverything);
    }
}

/// There is no folder worth showing.
fn no_folder(ui: &mut egui::Ui, nothing: &Nothing, asked: &mut Option<Asked>) {
    ui.heading("No photographs here");
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        // Centred by the caller's vertical_centered, so the row is laid out
        // from the middle rather than from the left edge.
        ui.add_space((ui.available_width() - 220.0).max(0.0) / 2.0);

        if ui
            .button("Open a folder…")
            .on_hover_text("Every photograph in it becomes the collection")
            .clicked()
        {
            *asked = Some(Asked::OpenFolder);
        }

        if ui
            .button("Open files…")
            .on_hover_text("Opens the folder they are in, on the first of them")
            .clicked()
        {
            *asked = Some(Asked::OpenFiles);
        }
    });

    if !nothing.recent.is_empty() {
        ui.add_space(18.0);
        ui.weak("Lately");
        ui.add_space(4.0);

        for folder in nothing.recent.iter().take(OFFERED) {
            let name = folder
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| folder.display().to_string());

            if ui
                .link(name)
                .on_hover_text(folder.display().to_string())
                .clicked()
            {
                *asked = Some(Asked::Open(folder.to_path_buf()));
            }
        }
    }

    if nothing.say_the_keys {
        ui.add_space(20.0);
        ui.weak("? for the keys · right-click a photograph for what can be done to it");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The list is short on purpose: a home screen, not a history.
    #[test]
    fn only_a_few_folders_are_offered() {
        assert_eq!(OFFERED, 6);
    }

    /// The two states are different screens, and the filtered one offers the
    /// command that undoes what emptied the folder.
    #[test]
    fn a_filtered_folder_is_not_an_empty_one() {
        let filtered = Nothing {
            filtered: true,
            rules: vec!["3 stars and better".into()],
            recent: Vec::new(),
            say_the_keys: false,
        };

        assert!(filtered.filtered);
        assert_eq!(filtered.rules.len(), 1);
    }
}

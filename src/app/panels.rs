//! The chrome around the views: menu bar, metadata panel, cache readout.

use eframe::egui::{self, RichText};

use crate::app::mode::Mode;
use crate::cache::StoreStats;
use crate::metadata::Metadata;

/// Something picked from the menu bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuAction {
    OpenFolder,
    OpenFiles,
    /// Switch what the window is for.
    Mode(Mode),
}

/// Draws the menu bar, returning what the user picked.
pub fn top_menu(ctx: &egui::Context, visible: bool, mode: Mode) -> Option<MenuAction> {
    let mut action = None;

    egui::TopBottomPanel::top("menu")
        .show_separator_line(false)
        .show_animated(ctx, visible, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    for (label, picked) in [
                        ("Open Folder", MenuAction::OpenFolder),
                        ("Open Files", MenuAction::OpenFiles),
                    ] {
                        if ui.button(label).clicked() {
                            action = Some(picked);
                            ui.close();
                        }
                    }
                });

                ui.menu_button("Mode", |ui| {
                    for wanted in Mode::ALL {
                        // Radio rather than plain buttons: the menu is also
                        // where the user finds out which mode they are in.
                        if ui.radio(mode == *wanted, wanted.label()).clicked() {
                            action = Some(MenuAction::Mode(*wanted));
                            ui.close();
                        }
                    }
                });
            });
        });

    action
}

/// Draws the metadata of the open image, in the order the configuration lists.
pub fn metadata_panel(ui: &mut egui::Ui, metadata: Option<&Metadata>, tags: &[String]) {
    ui.add_space(20.);
    ui.label(RichText::new("Image Metadata").heading());
    ui.add_space(10.);

    let Some(metadata) = metadata else {
        ui.label("Loading…");
        return;
    };

    for tag in tags {
        let Some(value) = metadata.tags.get(tag) else {
            continue;
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{tag}:")).strong());
            ui.label(value);
        });
    }
}

/// Draws how full the caches are, so the effect of the budgets is visible.
pub fn cache_stats(ui: &mut egui::Ui, images: &StoreStats, thumbnails: &StoreStats) {
    ui.add_space(20.);
    ui.label(RichText::new("Cache").heading());
    ui.add_space(10.);

    for (label, stats) in [("Images", images), ("Thumbnails", thumbnails)] {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{label}:")).strong());
            ui.label(format!(
                "{}/{} in RAM • {} on GPU",
                stats.in_ram, stats.total, stats.on_gpu
            ));
        });
    }

    if images.at_full_resolution > 0 {
        ui.label(format!(
            "{} ready to zoom into at full resolution",
            images.at_full_resolution
        ));
    }

    ui.label(format!(
        "{} of {} budget",
        format_mib(images.resident_bytes + thumbnails.resident_bytes),
        format_mib(images.budget_bytes + thumbnails.budget_bytes)
    ));

    if images.failed > 0 {
        ui.label(format!("{} image(s) could not be opened", images.failed));
    }
}

fn format_mib(bytes: usize) -> String {
    format!("{:.0} MiB", bytes as f64 / (1024.0 * 1024.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_are_reported_in_mebibytes() {
        assert_eq!(format_mib(0), "0 MiB");
        assert_eq!(format_mib(1024 * 1024), "1 MiB");
        assert_eq!(format_mib(1536 * 1024 * 1024), "1536 MiB");
    }
}

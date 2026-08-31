//! The chrome around the views: menu bar, metadata panel, cache readout.

use eframe::egui::{self, RichText};

use crate::app::mode::Mode;
use crate::cache::StoreStats;
use crate::config::{Motion, SlideshowConfig};
use crate::metadata::Metadata;

/// Something picked from the menu bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuAction {
    OpenFolder,
    OpenFiles,
    /// Switch what the window is for.
    Mode(Mode),
    /// Send every rejected picture in the folder to the bin.
    BinRejected,
    /// Open the editor for the keyboard map.
    Keyboard,
    /// Open the slideshow settings.
    Slideshow,
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

                    ui.separator();

                    if ui
                        .button("Send rejected to the bin…")
                        .on_hover_text("Every picture in this folder marked with X")
                        .clicked()
                    {
                        action = Some(MenuAction::BinRejected);
                        ui.close();
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

                ui.menu_button("Settings", |ui| {
                    if ui.button("Keyboard…").clicked() {
                        action = Some(MenuAction::Keyboard);
                        ui.close();
                    }

                    if ui.button("Slideshow…").clicked() {
                        action = Some(MenuAction::Slideshow);
                        ui.close();
                    }
                });
            });
        });

    action
}

/// Draws the slideshow settings, returning whether anything changed.
pub fn slideshow_settings(
    ctx: &egui::Context,
    open: &mut bool,
    config: &mut SlideshowConfig,
) -> bool {
    let mut changed = false;

    egui::Window::new("Slideshow")
        .open(open)
        .default_width(420.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Hold each picture for:");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut config.seconds_per_image)
                            .range(1..=600)
                            .suffix(" s"),
                    )
                    .changed();
            });

            ui.add_space(6.0);
            ui.label("While it is up:");

            for motion in Motion::ALL {
                let picked = ui.radio_value(&mut config.motion, *motion, motion.label());
                changed |= picked.changed();

                ui.indent(("motion", motion.label()), |ui| {
                    ui.weak(motion.description());
                });
            }

            if config.motion == Motion::Zoom {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("Creep closer by:");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut config.percent_zoom)
                                .range(0.0..=200.0)
                                .suffix(" %"),
                        )
                        .changed();
                });
            }

            ui.add_space(6.0);
            ui.weak("The arrow keys still work; moving by hand restarts the clock.");
        });

    changed
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
            // Truncated rather than wrapped, with the whole of it on hover:
            // one long value — the directory, nearly always — used to decide
            // how wide the whole panel was.
            ui.add(egui::Label::new(value).truncate())
                .on_hover_text(value);
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

    ui.add_space(6.0);
    memory(ui, images, thumbnails);

    if images.failed > 0 {
        ui.label(format!("{} image(s) could not be opened", images.failed));
    }
}

/// What the viewer is actually holding, tier by tier.
///
/// Tier by tier because one number was misleading: it counted the decoded
/// pixels in RAM and nothing else, while the textures on the adapter — the
/// same pixels again, plus a third for the mip chain — the camera thumbnails
/// standing in for them, and a folder's worth of metadata all went
/// unmentioned. On a large folder the figure shown was a fraction of what the
/// process was using.
fn memory(ui: &mut egui::Ui, images: &StoreStats, thumbnails: &StoreStats) {
    let rows = [
        (
            "Decoded",
            images.resident_bytes + thumbnails.resident_bytes,
            Some(images.budget_bytes + thumbnails.budget_bytes),
        ),
        (
            "On the GPU",
            images.gpu_bytes + thumbnails.gpu_bytes,
            Some(images.gpu_budget_bytes + thumbnails.gpu_budget_bytes),
        ),
        (
            "Thumbnails standing in",
            images.preview_bytes + thumbnails.preview_bytes,
            None,
        ),
        (
            "Metadata read ahead",
            images.scanned_bytes + thumbnails.scanned_bytes,
            Some(images.scanned_budget_bytes + thumbnails.scanned_budget_bytes),
        ),
    ];

    for (label, held, budget) in rows {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{label}:")).weak());
            ui.label(match budget {
                Some(budget) => format!("{} of {}", format_mib(held), format_mib(budget)),
                None => format_mib(held),
            });
        });
    }

    ui.add_space(2.0);
    ui.label(
        RichText::new(format!(
            "{} held in all",
            format_mib(images.held_bytes() + thumbnails.held_bytes())
        ))
        .strong(),
    );
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

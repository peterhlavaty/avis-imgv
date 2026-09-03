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
    /// Send every rejected picture in the folder to the bin.
    BinRejected,
    /// Open the viewer's own bin, which is a folder like any other.
    OpenBin,
    /// Delete everything in it, folder and all. Asked about first.
    EmptyBin,
    /// Open the whole settings window.
    AllSettings,
    /// Open the editor for the keyboard map.
    Keyboard,
    /// Open the slideshow settings.
    Slideshow,
    /// The glance-at list of what the keys currently are.
    CheatSheet,
    /// The legend for the glyphs, badges and overlay colours.
    MarksLegend,
    /// What may go in a name template, and what each placeholder expands to.
    Placeholders,
    /// Everything the viewer has said lately, whether or not it was seen.
    Messages,
    OpenConfigFile,
    OpenLogFile,
    OpenManual,
    About,
}

/// The keys the menu names beside its own rows.
///
/// Rendered from the bindings rather than written into the labels, so a rebind
/// stays correct: Microsoft asks for exactly that of a menu that names a key.
#[derive(Debug, Clone, Default)]
pub struct MenuKeys {
    pub cheat_sheet: String,
    pub settings: String,
}

/// Draws the menu bar, returning what the user picked.
pub fn top_menu(
    ctx: &egui::Context,
    visible: bool,
    mode: Mode,
    keys: &MenuKeys,
) -> Option<MenuAction> {
    let mut action = None;

    egui::TopBottomPanel::top("menu")
        .show_separator_line(false)
        .show_animated(ctx, visible, |ui| {
            // Nothing on the bar while a window is in front. Opening a folder
            // or changing the mode from behind a settings window is a click
            // aimed past the window it was meant for.
            if crate::utils::is_a_window_in_front(ui.ctx()) {
                ui.disable();
            }

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

                    ui.separator();

                    // The bin is a folder, so the way to look in it is the way
                    // into any other folder: open it. That is the whole
                    // argument for it being a folder.
                    if ui
                        .button("Open the bin")
                        .on_hover_text(
                            "The viewer's own bin, as a folder — what an hour of \
                             culling threw away, before any of it is really gone",
                        )
                        .clicked()
                    {
                        action = Some(MenuAction::OpenBin);
                        ui.close();
                    }

                    if ui
                        .button("Empty the bin…")
                        .on_hover_text("Delete everything in it for good")
                        .clicked()
                    {
                        action = Some(MenuAction::EmptyBin);
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

                // Between what the window is for and how it is set up, which
                // is where it sits: the panels are neither a mode nor a
                // setting but what is on screen right now. The rows are
                // `ui::panel`'s, so this menu and the Show submenu on the
                // photograph cannot come to say different things.
                ui.menu_button("View", crate::ui::panel::show_and_hide);

                ui.menu_button("Settings", |ui| {
                    // The third entry on a menu that has had two since it was
                    // written. Keyboard and Slideshow stay as deep links to
                    // two of the eleven pages, because they are the only
                    // settings routes anybody has learned.
                    if ui
                        .button(format!("All settings…  {}", keys.settings))
                        .on_hover_text("Every setting the viewer has, with a search box")
                        .clicked()
                    {
                        action = Some(MenuAction::AllSettings);
                        ui.close();
                    }

                    ui.separator();

                    if ui
                        .button("Keyboard…")
                        .on_hover_text("Every key the viewer reads, and what it does")
                        .clicked()
                    {
                        action = Some(MenuAction::Keyboard);
                        ui.close();
                    }

                    if ui
                        .button("Slideshow…")
                        .on_hover_text("How long each picture is held, and whether it moves")
                        .clicked()
                    {
                        action = Some(MenuAction::Slideshow);
                        ui.close();
                    }
                });

                help_menu(ui, keys, &mut action);
            });

            crate::ui::panel::menu(ui, &MENU_BAR, |_| {});
        });

    action
}

/// The Help menu.
///
/// The menu bar was three menus and eleven items with no Help at all, so the
/// cheat sheet, the configuration file and the log were reachable only by
/// somebody who already knew they existed.
fn help_menu(ui: &mut egui::Ui, keys: &MenuKeys, action: &mut Option<MenuAction>) {
    ui.menu_button("Help", |ui| {
        let rows: [(String, &str, MenuAction); 4] = [
            (
                format!("Keys…  {}", keys.cheat_sheet),
                "What every key does in the mode on screen",
                MenuAction::CheatSheet,
            ),
            (
                "Keyboard…".to_string(),
                "Change what a key does",
                MenuAction::Keyboard,
            ),
            (
                "What the marks mean".to_string(),
                "The glyphs on a stack, the badges on a cell and the overlay colours",
                MenuAction::MarksLegend,
            ),
            (
                "Template placeholders…".to_string(),
                "What may go in a name template, and what each one expands to",
                MenuAction::Placeholders,
            ),
        ];

        for (label, hint, picked) in rows {
            if ui.button(label).on_hover_text(hint).clicked() {
                *action = Some(picked);
                ui.close();
            }
        }

        ui.separator();

        if ui
            .button("Recent messages…")
            .on_hover_text("Everything the viewer has said lately, whether or not it was seen")
            .clicked()
        {
            *action = Some(MenuAction::Messages);
            ui.close();
        }

        ui.separator();

        let rows = [
            (
                "Open the configuration file",
                "Every setting, as JSON",
                MenuAction::OpenConfigFile,
            ),
            (
                "Open the log file",
                "What the viewer has been doing, and what went wrong",
                MenuAction::OpenLogFile,
            ),
            (
                "Open the manual",
                "The README, in a browser",
                MenuAction::OpenManual,
            ),
            (
                "About",
                "Which build this is, what it is drawing on, and where its files are",
                MenuAction::About,
            ),
        ];

        for (label, hint, picked) in rows {
            if ui.button(label).on_hover_text(hint).clicked() {
                *action = Some(picked);
                ui.close();
            }
        }
    });
}

/// What the metadata panel says for itself.
///
/// Here rather than beside the `SidePanel` that carries it, because the rows
/// drawn in it are this file's and the settings row the menu ends on is the
/// list of which tags they are.
pub const METADATA_PANEL: crate::ui::panel::Chrome<'static> = crate::ui::panel::Chrome {
    subject: crate::ui::surface::Subject::the("The metadata panel"),
    hide: Some(crate::app::input::Command::ToggleSidePanel),
    key: Some("general.sc_toggle_side_panel"),
    page: crate::config::registry::Page::ThePhotograph,
    setting: "general.metadata_tags",
};

/// And what the menu bar says for itself.
pub const MENU_BAR: crate::ui::panel::Chrome<'static> = crate::ui::panel::Chrome {
    subject: crate::ui::surface::Subject::the("The menu bar"),
    hide: Some(crate::app::input::Command::ToggleMenu),
    key: Some("general.sc_menu"),
    page: crate::config::registry::Page::TheWindow,
    setting: "general.panels_at_start",
};

/// Draws the metadata of the open image, in the order the configuration lists.
///
/// `open` says whether there is a photograph at all: with no folder the panel
/// said "Loading…", which is a lie that never resolves.
///
/// Returns the settings row a menu asked for, if one did.
pub fn metadata_panel(
    ui: &mut egui::Ui,
    metadata: Option<&Metadata>,
    tags: &[String],
    open: bool,
) -> Option<&'static str> {
    let mut asked = None;
    ui.add_space(20.);
    ui.label(RichText::new("Image Metadata").heading());
    ui.add_space(10.);

    let Some(metadata) = metadata else {
        if open {
            ui.label("Reading it…");
        } else {
            ui.weak("No photograph open.");
        }
        return asked;
    };

    let mut drawn = 0;

    for tag in tags {
        let Some(value) = metadata.tags.get(tag) else {
            continue;
        };
        drawn += 1;

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{tag}:")).strong());
            // Truncated rather than wrapped, with the whole of it on hover:
            // one long value — the directory, nearly always — used to decide
            // how wide the whole panel was.
            let row = ui.add(
                egui::Label::new(value)
                    .truncate()
                    .sense(egui::Sense::click()),
            );

            // The tag names the row; the value is what the two copy verbs are
            // about, and the menu covers the row it was asked for.
            crate::ui::surface::with_menu(
                ui,
                &row,
                crate::ui::surface::Subject::of(tag, value),
                value,
                |ui| {
                    if ui.button("Copy the value").clicked() {
                        ui.ctx().copy_text(value.clone());
                        ui.close();
                    }
                    if ui.button("Copy the tag name").clicked() {
                        ui.ctx().copy_text(tag.clone());
                        ui.close();
                    }
                    if crate::ui::surface::more_settings(
                        ui,
                        crate::config::registry::Page::ThePhotograph,
                    ) {
                        asked = Some("general.metadata_tags");
                        ui.close();
                    }
                },
            );
        });
    }

    // A tag the configuration asks for that the file does not carry is skipped
    // in silence, so a list of eight that draws none looked like a failure.
    if drawn == 0 {
        ui.weak("This photograph carries none of the tags being asked for.");
    } else if drawn < tags.len() {
        ui.add_space(4.0);
        ui.weak(format!(
            "{} of {} asked for; the rest are not in this file.",
            drawn,
            tags.len()
        ));
    }

    asked
}

/// Draws how full the caches are, so the effect of the budgets is visible.
///
/// Returns the settings row a menu asked for. Every line here is a true
/// statement about a number somebody can change, and the page that holds it was
/// two menus and a scroll away.
pub fn cache_stats(
    ui: &mut egui::Ui,
    images: &StoreStats,
    thumbnails: &StoreStats,
) -> Option<&'static str> {
    let mut asked = None;

    ui.add_space(20.);
    let heading =
        ui.add(egui::Label::new(RichText::new("Cache").heading()).sense(egui::Sense::click()));

    crate::ui::surface::with_menu(
        ui,
        &heading,
        crate::ui::surface::Subject::the("The cache panel"),
        "What the viewer is holding, and what bounds it.",
        |ui| {
            if crate::ui::surface::more_settings(ui, crate::config::registry::Page::SpeedAndMemory)
            {
                asked = Some("cache.ram_budget_mb");
                ui.close();
            }
        },
    );
    ui.add_space(10.);

    for (label, stats, hint) in [
        (
            "Images",
            images,
            "Full size photographs. How many of the folder are decoded and waiting, \
             and how many of those have a texture on the graphics card.",
        ),
        (
            "Thumbnails",
            thumbnails,
            "The contact sheet's own copies, decoded small. They are kept whichever \
             view is on screen, so the sheet opens instantly.",
        ),
    ] {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{label}:")).strong());
            ui.label(format!(
                "{}/{} in RAM • {} on GPU",
                stats.in_ram, stats.total, stats.on_gpu
            ))
            .on_hover_text(hint);
        });
    }

    if images.at_full_resolution > 0 {
        ui.label(format!(
            "{} ready to zoom into at full resolution",
            images.at_full_resolution
        ))
        .on_hover_text(
            "Browsing keeps a copy no larger than the screen. These are the ones whose \
             own pixels are also in hand, so magnifying them costs nothing.",
        );
    }

    ui.add_space(6.0);
    memory(ui, images, thumbnails);

    if images.failed > 0 {
        let failed = ui.add(
            egui::Label::new(format!("{} image(s) could not be opened", images.failed))
                .sense(egui::Sense::click()),
        );

        let count = format!("{} photographs", images.failed);

        crate::ui::surface::with_menu(
            ui,
            &failed,
            crate::ui::surface::Subject::of("Would not open", &count),
            "The log says why for each of them.",
            |ui| {
                if ui.button("Open the log").clicked() {
                    if let Some(path) = crate::logging::path() {
                        crate::actions::reveal::with_the_system(&path);
                    }
                    ui.close();
                }
            },
        );
    }

    asked
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
            "Pixels in RAM, waiting to be drawn. Governed by the RAM budget; when it \
             is full the photograph furthest from the cursor is dropped.",
        ),
        (
            "On the GPU",
            images.gpu_bytes + thumbnails.gpu_bytes,
            Some(images.gpu_budget_bytes + thumbnails.gpu_budget_bytes),
            "Textures on the graphics card: the same pixels again, plus a third for \
             the mip chain. Governed by the GPU budget.",
        ),
        (
            "Thumbnails standing in",
            images.preview_bytes + thumbnails.preview_bytes,
            None,
            "The camera's own embedded preview, kept so a photograph that has not \
             been decoded yet still shows something.",
        ),
        (
            "Metadata read ahead",
            images.scanned_bytes + thumbnails.scanned_bytes,
            Some(images.scanned_budget_bytes + thumbnails.scanned_budget_bytes),
            "EXIF and XMP read from the same buffer as the decode, so the panel and \
             the filter never wait on a file.",
        ),
    ];

    for (label, held, budget, hint) in rows {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{label}:")).weak());
            ui.label(match budget {
                Some(budget) => format!("{} of {}", format_mib(held), format_mib(budget)),
                None => format_mib(held),
            })
            .on_hover_text(hint);
        });
    }

    ui.add_space(2.0);
    ui.label(
        RichText::new(format!(
            "{} held in all",
            format_mib(images.held_bytes() + thumbnails.held_bytes())
        ))
        .strong(),
    )
    .on_hover_text(
        "The four rows above added up. One figure used to be shown and it counted \
         only the first, so on a large folder it was a fraction of what was really held.",
    );
}

fn format_mib(bytes: usize) -> String {
    format!("{:.0} MiB", bytes as f64 / (1024.0 * 1024.0))
}

/// One line saying the keyboard has been taken, and which key gets it back.
///
/// `are_inputs_muted` is `a window in front || memory.focused().is_some()`, so
/// the whole viewer goes deaf while any text field holds focus — the filter bar's
/// three, the tag panel's one, the folder jobs' eight — with `Escape` the only
/// way out and `Alt+Q` the only shortcut that survives. Nothing on screen said
/// any of it, so the symptom was a viewer that had stopped answering its keys.
pub fn typing_notice(ctx: &egui::Context) {
    // Not while a window is in front. The viewer being deaf is the point
    // there, not a surprise to be explained, and the line would be drawn at
    // the foot of a window nobody is typing into.
    if crate::utils::is_a_window_in_front(ctx) {
        return;
    }

    if !ctx.memory(|memory| memory.focused().is_some()) {
        return;
    }

    egui::TopBottomPanel::bottom("typing")
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Typing — Escape to get the keys back.")
                        .color(ui.visuals().warn_fg_color),
                );
            });
        });
}

/// One line, on the first session only, naming the two keys nothing else does.
///
/// Not a tour and not a dialogue: two keys in the corner, gone as soon as
/// either is pressed. Somebody who already knows the program never sees it.
pub fn first_run_hint(ctx: &egui::Context, menu_key: &str) {
    egui::TopBottomPanel::bottom("first run")
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.weak(format!("Press ? for the keys Â· {menu_key} for the menu"));
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bar names the panels at the top level, beside what the window is
    /// for and how it is set up.
    #[test]
    fn the_bar_carries_a_view_menu() {
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            let _ = top_menu(ctx, true, Mode::Image, &MenuKeys::default());
        });

        let drawn: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) => Some(text.galley.text().to_string()),
                _ => None,
            })
            .collect();

        for menu in ["File", "Mode", "View", "Settings", "Help"] {
            assert!(drawn.iter().any(|text| text == menu), "{menu}: {drawn:?}");
        }
    }

    #[test]
    fn sizes_are_reported_in_mebibytes() {
        assert_eq!(format_mib(0), "0 MiB");
        assert_eq!(format_mib(1024 * 1024), "1 MiB");
        assert_eq!(format_mib(1536 * 1024 * 1024), "1536 MiB");
    }
}

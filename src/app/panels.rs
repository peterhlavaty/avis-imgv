//! The chrome around the views: menu bar, metadata panel, cache readout.

use eframe::egui::{self, RichText};

use crate::cache::StoreStats;
use crate::metadata::Metadata;
use crate::mode::Mode;
use crate::ui::keys;

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
    /// Open the whole settings card.
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

impl MenuAction {
    /// Whether carrying this out means the cards are done with.
    ///
    /// The bar answers the mouse from over a card now, so it can be asked for
    /// two different kinds of thing while one is up. Opening another card is
    /// one — the deck switches, and the way back is its own bar. Anything about
    /// the folder or the mode is the other: somebody who has just asked for a
    /// different folder is asking to look at photographs, and leaving the
    /// settings over the top of them is answering a question nobody asked.
    ///
    /// The two files and the manual are neither. They open outside the viewer
    /// and change nothing in it, and a person who opens the configuration file
    /// from the settings is coming straight back to the settings.
    pub(super) fn goes_back_to_the_photographs(self) -> bool {
        match self {
            MenuAction::OpenFolder
            | MenuAction::OpenFiles
            | MenuAction::Mode(_)
            | MenuAction::BinRejected
            | MenuAction::OpenBin
            | MenuAction::EmptyBin => true,

            MenuAction::AllSettings
            | MenuAction::Keyboard
            | MenuAction::Slideshow
            | MenuAction::CheatSheet
            | MenuAction::MarksLegend
            | MenuAction::Placeholders
            | MenuAction::Messages
            | MenuAction::About
            | MenuAction::OpenConfigFile
            | MenuAction::OpenLogFile
            | MenuAction::OpenManual => false,
        }
    }
}

/// Draws the menu bar, returning what the user picked.
///
/// The keys it names beside its rows are `ui::keys`', published once a frame.
/// Two of them were threaded in as strings while two rows were all that named
/// one; every menu in the program names them now.
///
/// The bar stays on screen while a card is up, and answers the mouse there:
/// **Help → About** from the settings is one click rather than Escape and a
/// menu. That costs a second layer, because a panel lives in
/// `LayerId::background()` and cannot be moved out of it, and egui's modal
/// layer — which is what stops the photograph behind a card from being clicked
/// — is decided by comparing `Order`, so everything in every panel is under
/// every card there is. So while a card is up the bar is laid out in its panel
/// for the height alone and drawn for real in an `Area` above the card, in the
/// same place, from the same function. Two layouts of five buttons a frame,
/// against a menu bar nobody can press.
pub fn top_menu(ctx: &egui::Context, visible: bool, mode: Mode) -> Option<MenuAction> {
    let mut action = None;
    let over_a_card = crate::ui::front::is_in_front(ctx);

    // Where the bar's contents came to inside the panel, so the copy above the
    // card lands on the same pixels: the panel's own rectangle includes its
    // margin, and drawing at that corner would shift the bar the moment a card
    // opened.
    let mut at = egui::Rect::NOTHING;

    egui::TopBottomPanel::top("menu")
        .show_separator_line(false)
        .show_animated(ctx, visible, |ui| {
            // Laid out and not painted: `set_invisible` also disables, so the
            // ids it takes are never the ones a click lands on and the bar's
            // own right-click menu stays with the copy that can be reached.
            if over_a_card {
                ui.set_invisible();
            }

            let corner = ui.max_rect().min;
            let width = ui.available_width();

            action = rows(ui, mode);

            at = egui::Rect::from_min_size(corner, egui::vec2(width, ui.min_rect().height()));
        });

    if !over_a_card || !at.is_positive() {
        return action;
    }

    egui::Area::new(egui::Id::new("the menu bar over a card"))
        .order(egui::Order::Foreground)
        .fixed_pos(at.min)
        .constrain_to(at)
        .show(ctx, |ui| {
            // The clip rectangle is what `ui::panel::menu` asks the pointer
            // about, so the bar's own menu opens over the bar and nowhere else.
            ui.set_clip_rect(at);
            ui.set_min_size(at.size());
            ui.set_max_size(at.size());

            // Salted: the copy in the panel has taken every one of these ids
            // already, and two widgets on one id is a warning painted over the
            // bar in a debug build.
            ui.push_id("over a card", |ui| {
                action = rows(ui, mode);
            });
        });

    action
}

/// The bar itself, drawn wherever it has been given room.
fn rows(ui: &mut egui::Ui, mode: Mode) -> Option<MenuAction> {
    let mut picked = None;

    ui.horizontal(|ui| {
        ui.menu_button("File", |ui| {
            for (label, wanted) in [
                ("Open Folder", MenuAction::OpenFolder),
                ("Open Files", MenuAction::OpenFiles),
            ] {
                if ui.button(label).clicked() {
                    picked = Some(wanted);
                    ui.close();
                }
            }

            ui.separator();

            if ui
                .button("Send rejected to the bin…")
                .on_hover_text("Every picture in this folder marked with X")
                .clicked()
            {
                picked = Some(MenuAction::BinRejected);
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
                picked = Some(MenuAction::OpenBin);
                ui.close();
            }

            if ui
                .button("Empty the bin…")
                .on_hover_text("Delete everything in it for good")
                .clicked()
            {
                picked = Some(MenuAction::EmptyBin);
                ui.close();
            }
        });

        ui.menu_button("Mode", |ui| {
            for wanted in Mode::ALL {
                // Radio rather than plain buttons: the menu is also
                // where the user finds out which mode they are in.
                if keys::radio(ui, mode == *wanted, wanted.label(), wanted.key()).clicked() {
                    picked = Some(MenuAction::Mode(*wanted));
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
            if keys::button(ui, "All settings…", "general.sc_settings")
                .on_hover_text("Every setting the viewer has, with a search box")
                .clicked()
            {
                picked = Some(MenuAction::AllSettings);
                ui.close();
            }

            ui.separator();

            if ui
                .button("Keyboard…")
                .on_hover_text("Every key the viewer reads, and what it does")
                .clicked()
            {
                picked = Some(MenuAction::Keyboard);
                ui.close();
            }

            if ui
                .button("Slideshow…")
                .on_hover_text("How long each picture is held, and whether it moves")
                .clicked()
            {
                picked = Some(MenuAction::Slideshow);
                ui.close();
            }
        });

        help_menu(ui, &mut picked);
    });

    crate::ui::panel::menu(ui, &MENU_BAR, |_| {});

    picked
}

/// The Help menu.
///
/// The menu bar was three menus and eleven items with no Help at all, so the
/// cheat sheet, the configuration file and the log were reachable only by
/// somebody who already knew they existed.
fn help_menu(ui: &mut egui::Ui, action: &mut Option<MenuAction>) {
    ui.menu_button("Help", |ui| {
        // The path is empty where the row is not something a key also does,
        // which is what `keys::of` answers with nothing.
        let rows: [(&str, &str, &str, MenuAction); 4] = [
            (
                "Keys…",
                "fixed.cheat_sheet",
                "What every key does in the mode on screen",
                MenuAction::CheatSheet,
            ),
            (
                "Keyboard…",
                "",
                "Change what a key does",
                MenuAction::Keyboard,
            ),
            (
                "What the marks mean",
                "",
                "The glyphs on a stack, the badges on a cell and the overlay colours",
                MenuAction::MarksLegend,
            ),
            (
                "Template placeholders…",
                "",
                "What may go in a name template, and what each one expands to",
                MenuAction::Placeholders,
            ),
        ];

        for (label, path, hint, picked) in rows {
            if keys::button(ui, label, path).on_hover_text(hint).clicked() {
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
    hide: Some(crate::command::Command::ToggleSidePanel),
    key: Some("general.sc_toggle_side_panel"),
    page: crate::config::registry::Page::ThePhotograph,
    setting: "general.metadata_tags",
};

/// And what the menu bar says for itself.
pub const MENU_BAR: crate::ui::panel::Chrome<'static> = crate::ui::panel::Chrome {
    subject: crate::ui::surface::Subject::the("The menu bar"),
    hide: Some(crate::command::Command::ToggleMenu),
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
/// `are_inputs_muted` is `something in front || memory.focused().is_some()`, so
/// the whole viewer goes deaf while any text field holds focus — the filter bar's
/// three, the tag panel's one, the folder jobs' eight — with `Escape` the only
/// way out and `Alt+Q` the only shortcut that survives. Nothing on screen said
/// any of it, so the symptom was a viewer that had stopped answering its keys.
pub fn typing_notice(ctx: &egui::Context) {
    // Not while a card is in front. The viewer being deaf is the point
    // there, not a surprise to be explained, and the line would be drawn at
    // the foot of a window nobody is typing into.
    if crate::ui::front::is_in_front(ctx) {
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
            let _ = top_menu(ctx, true, Mode::Image);
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

    /// The bar answers the mouse from over a card.
    ///
    /// It used to draw itself disabled while anything of the viewer's own was
    /// up, because a panel is in `LayerId::background()` and every card is
    /// above it: **Help → About** from the settings meant Escape first. What
    /// this asserts is the two halves of the fix — that the copy above the
    /// card is drawn in the same place as the panel's own, and that a press on
    /// it is heard.
    #[test]
    fn the_bar_is_pressed_from_over_a_card() {
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 800.0));
        let quiet = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };

        // Where **Help** lands with nothing in front, and where it lands with
        // a card in front. The same place, or the bar jumps as a card opens.
        let plain = egui::Context::default();
        let under = crate::ui::drawn::text_at(
            &plain.run(quiet.clone(), |ctx| {
                let _ = top_menu(ctx, true, Mode::Image);
            }),
            "Help",
        )
        .expect("the bar is drawn");

        let ctx = egui::Context::default();
        let frame = |input: egui::RawInput| {
            let mut action = None;
            let output = ctx.run(input, |ctx| {
                crate::ui::front::set_in_front(ctx, true);
                action = top_menu(ctx, true, Mode::Image);
            });

            (output, action)
        };

        // Twice: the copy above the card is an `Area`, and egui lays an area
        // whose size it does not yet know out in a pass that paints nothing.
        let _ = frame(quiet.clone());
        let (output, action) = frame(quiet.clone());
        assert_eq!(action, None);

        let over = crate::ui::drawn::text_at(&output, "Help").expect("the copy is drawn");
        assert_eq!(over, under);

        // And the press is heard, which under the card it was not.
        let (_, action) = frame(egui::RawInput {
            screen_rect: Some(screen),
            events: vec![
                egui::Event::PointerMoved(over),
                egui::Event::PointerButton {
                    pos: over,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: over,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        });

        // The Help menu opens rather than reporting anything, so what is
        // asserted is that its rows are now on screen.
        assert_eq!(action, None);
        let (output, _) = frame(quiet);
        let drawn = crate::ui::drawn::text(&output);
        assert!(
            drawn.iter().any(|text| text.contains("Recent messages")),
            "the Help menu did not open: {drawn:?}"
        );
    }

    /// Every action is one or the other, and the compiler says so; this is the
    /// two that are easiest to get wrong.
    #[test]
    fn a_folder_leaves_the_cards_and_another_card_does_not() {
        assert!(MenuAction::OpenFolder.goes_back_to_the_photographs());
        assert!(MenuAction::Mode(Mode::Grid).goes_back_to_the_photographs());
        assert!(!MenuAction::About.goes_back_to_the_photographs());
        assert!(!MenuAction::AllSettings.goes_back_to_the_photographs());
        assert!(!MenuAction::OpenLogFile.goes_back_to_the_photographs());
    }

    #[test]
    fn sizes_are_reported_in_mebibytes() {
        assert_eq!(format_mib(0), "0 MiB");
        assert_eq!(format_mib(1024 * 1024), "1 MiB");
        assert_eq!(format_mib(1536 * 1024 * 1024), "1536 MiB");
    }
}

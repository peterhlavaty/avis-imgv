//! What every panel does about the second button, written once.
//!
//! Everything drawn answers the second button, and a panel is the largest
//! thing drawn — but until now it was the only surface that answered in some
//! of its parts and not others. The rows in the history answered; the heading
//! above them did not, nor the strip of blank to the right of a short row, nor
//! the half of the panel below the last one, nor anything at all in the
//! keyword panel, the strip, the filter bar or the metadata panel. A button
//! that works in some of a panel and not the rest is worse than one that never
//! works: it teaches a person the panel has no menu, and they stop pressing.
//!
//! Two things make that hard to do panel by panel, and are the reason this is
//! one file rather than seven copies.
//!
//! The first is that most of a panel is not a widget. A heading, a separator
//! and the gaps between rows are painted, not sensed, so there is no response
//! to hang a menu on. Laying a click-sensing rectangle over the whole panel
//! does not answer it either: registered before the contents it is hidden by
//! the drag-to-scroll rectangle every scroll area lays over itself once its
//! content has outgrown it — egui hands the click to nothing when a drag-only
//! widget is on top of a click-only one — and registered after them it takes
//! the press away from every button in the panel. So the panel reads the press
//! itself, from the pointer and its own rectangle, and
//! [`crate::ui::surface::menu_when`] takes the answer.
//!
//! The second is that reading it for itself means knowing whether something
//! *inside* the panel answered first, which is what `surface::taken` is for.
//! The panel is drawn last, over all of its contents, and stands down when one
//! of them has taken the press.
//!
//! What each panel then has to say for itself is five things — what it is,
//! what puts it away, which key does that, and where its settings live — and
//! nothing else. The rows, their wording, their order and the route back to
//! the program are here.

use std::sync::Mutex;

use eframe::egui;

use crate::app::input::Command;
use crate::config::registry::Page;
use crate::ui::surface::{self, Subject};

/// What a panel's own menu asked the program for.
///
/// A mailbox rather than a return value, for the reason the rails have one:
/// panels are drawn from four subsystems, three of which have neither the
/// configuration nor the command dispatcher in hand, and threading an answer
/// back through `GridView::filmstrip` or `bottom_bar::ui` would be five new
/// arguments to say one thing. `App::take_panel_ask` empties it once a frame,
/// through the same routes the settings window uses — which is what puts a
/// panel put away from its own menu into the history for free.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ask {
    /// Put the panel away.
    Hide(Command),
    /// Open the settings window at this row.
    Settings(&'static str),
    /// Open the keyboard editor with this row armed.
    BindAKey(&'static str),
}

static ASKED: Mutex<Option<Ask>> = Mutex::new(None);

/// What a panel's menu asked for, if anything, taking the ask.
pub fn asked() -> Option<Ask> {
    ASKED.lock().ok().and_then(|mut asked| asked.take())
}

/// Leaves an ask for whoever empties the mailbox.
fn ask(what: Ask) {
    if let Ok(mut asked) = ASKED.lock() {
        *asked = Some(what);
    }
}

/// What one panel says for itself.
///
/// Everything else about a panel's menu — which rows, in what order, worded
/// how — is the same for all of them and is not here.
pub struct Chrome<'a> {
    /// What the menu is about, drawn at the top of it.
    pub subject: Subject<'a>,
    /// What puts the panel away. `None` for a panel that cannot be put away,
    /// which is the status bar and nothing else: the menu still opens, still
    /// says what it is about and still leads to the settings that govern it.
    pub hide: Option<Command>,
    /// The key row that shows and hides it, so a person can bind one from the
    /// panel rather than from the keyboard editor. `None` where the key is not
    /// a setting.
    pub key: Option<&'static str>,
    /// The settings page the menu ends on, and the row it opens there.
    pub page: Page,
    pub setting: &'static str,
}

/// The rows every menu drawn inside a panel ends with.
///
/// Public because a menu opened on a *row* of a panel is the same menu as far
/// as the panel is concerned — the history's rows have carried these two since
/// they were written — and two copies would be two things to keep in step.
pub fn rows(ui: &mut egui::Ui, chrome: &Chrome<'_>) {
    if let Some(hide) = chrome.hide {
        if ui
            .button("Hide this panel")
            .on_hover_text("Nothing in it is lost; this only puts it away.")
            .clicked()
        {
            ask(Ask::Hide(hide));
            ui.close();
        }
    }

    if let Some(key) = chrome.key {
        if surface::bind_a_key(ui, "showing and hiding it") {
            ask(Ask::BindAKey(key));
            ui.close();
        }
    }

    if surface::more_settings(ui, chrome.page) {
        ask(Ask::Settings(chrome.setting));
        ui.close();
    }
}

/// The menu the panel itself carries, over the whole of it.
///
/// Called last inside the panel's closure, after everything the panel holds,
/// so that anything in it with a menu of its own answers first.
pub fn menu(ui: &mut egui::Ui, chrome: &Chrome<'_>, extra: impl FnOnce(&mut egui::Ui)) {
    let pressed = wanted(ui);

    // A rectangle nothing can hit. The press has already been decided, and the
    // popup wants a response only for an identity and a layer to live in — a
    // real rectangle the size of a panel would be a surface that hovers
    // instead of the things drawn in it.
    let response = ui.interact(
        egui::Rect::NOTHING,
        ui.id().with("the panel itself"),
        egui::Sense::hover(),
    );

    surface::menu_when(ui, &response, chrome.subject, pressed, |ui| {
        ui.set_max_width(surface::WIDEST);
        extra(ui);
        rows(ui, chrome);
    });
}

/// Whether the panel itself was asked for a menu.
fn wanted(ui: &egui::Ui) -> bool {
    // A panel under a window in front draws itself disabled, and every one of
    // its verbs changes what is behind the window being read.
    if !ui.is_enabled() {
        return false;
    }

    // The panel's own rectangle, and the pointer actually over *it*: a menu
    // already open, an overlay or a window is a layer in front, and
    // `rect_contains_pointer` is the one reading that knows about both layers
    // and egui's modal one.
    if !ui.rect_contains_pointer(ui.clip_rect()) {
        return false;
    }

    if !ui
        .ctx()
        .input(|i| i.pointer.button_pressed(egui::PointerButton::Secondary))
    {
        return false;
    }

    // Whatever is drawn in the panel answers before the panel does.
    !surface::taken(ui.ctx())
}

/// Every panel in the program, in the order they are drawn.
///
/// Here so that a panel added without a menu fails a test rather than being
/// noticed by somebody pressing the second button on it, and so that the
/// paths each one carries are checked against the registry: a settings row or
/// a key row that has been renamed leaves a menu whose last two rows quietly
/// do nothing, which is the failure this list exists to catch.
#[cfg(test)]
const EVERY_PANEL: &[&Chrome<'static>] = &[
    &crate::app::panels::MENU_BAR,
    &crate::ui::perf_metrics::CHROME,
    &crate::ui::filter_bar::CHROME,
    &crate::app::panels::METADATA_PANEL,
    &crate::history::panel::CHROME,
    &crate::ui::tag_panel::CHROME,
    &crate::view::grid_view::filmstrip::CHROME,
    &crate::view::image_view::bottom_bar::CHROME,
];

#[cfg(test)]
mod tests {
    use super::*;

    fn a_chrome() -> Chrome<'static> {
        Chrome {
            subject: Subject::the("The history panel"),
            hide: Some(Command::ToggleHistoryPanel),
            key: Some("history.sc_panel"),
            page: Page::History,
            setting: "history.panel_visible",
        }
    }

    /// The mailbox holds one ask and gives it up once.
    #[test]
    fn an_ask_is_taken_once() {
        let _ = asked();

        ask(Ask::Hide(Command::ToggleFilmstrip));
        assert_eq!(asked(), Some(Ask::Hide(Command::ToggleFilmstrip)));
        assert_eq!(asked(), None);

        ask(Ask::Settings("history.panel_visible"));
        assert_eq!(asked(), Some(Ask::Settings("history.panel_visible")));
        assert_eq!(asked(), None);

        ask(Ask::BindAKey("history.sc_panel"));
        assert_eq!(asked(), Some(Ask::BindAKey("history.sc_panel")));
        assert_eq!(asked(), None);
    }

    /// The three rows, in the order every panel draws them.
    #[test]
    fn the_rows_are_the_same_three_everywhere() {
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rows(ui, &a_chrome());
            });
        });

        let drawn = painted(&output);
        let at = |what: &str| {
            drawn
                .iter()
                .position(|text| text.contains(what))
                .unwrap_or_else(|| panic!("{what} is drawn: {drawn:?}"))
        };

        assert!(at("Hide this panel") < at("Bind a key"));
        assert!(at("Bind a key") < at("More settings"));
    }

    /// A panel that cannot be put away still says what it is and where its
    /// settings are — the status bar is the one.
    #[test]
    fn a_panel_that_cannot_be_hidden_draws_the_rest() {
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rows(
                    ui,
                    &Chrome {
                        subject: Subject::the("The status bar"),
                        hide: None,
                        key: None,
                        page: Page::ThePhotograph,
                        setting: "image_view.name_format",
                    },
                );
            });
        });

        let drawn = painted(&output);
        assert!(!drawn.iter().any(|text| text.contains("Hide this panel")));
        assert!(!drawn.iter().any(|text| text.contains("Bind a key")));
        assert!(drawn.iter().any(|text| text.contains("More settings")));
    }

    /// The fault this file was written for: a right-click on the empty half of
    /// a panel opened nothing, because there is no widget there to open it.
    #[test]
    fn a_menu_opens_on_the_empty_part_of_a_panel() {
        assert!(menu_opened_on_a_right_click(egui::pos2(100.0, 180.0), 40));
        assert!(menu_opened_on_a_right_click(egui::pos2(100.0, 180.0), 0));
    }

    /// And the reason it cannot simply lay a rectangle over the panel: a
    /// button inside one still gets its own press, whether or not the list it
    /// is in has grown long enough to scroll.
    #[test]
    fn a_row_in_the_panel_answers_before_the_panel_does() {
        assert!(!menu_opened_on_a_right_click(egui::pos2(50.0, 10.0), 40));
        assert!(!menu_opened_on_a_right_click(egui::pos2(50.0, 10.0), 3));
    }

    /// Right-clicks at `at` in a panel holding `rows` rows in a scroll area,
    /// and answers whether the *panel's* menu opened.
    fn menu_opened_on_a_right_click(at: egui::Pos2, rows_in_it: usize) -> bool {
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 200.0));

        let draw = |ctx: &egui::Context| {
            egui::SidePanel::left("a panel")
                .exact_width(200.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(120.0)
                        .show(ui, |ui| {
                            for row in 0..rows_in_it {
                                let response = ui.selectable_label(false, format!("row {row}"));
                                surface::menu(ui, &response, Subject::the("A row"), |ui| {
                                    let _ = ui.button("Do only this again");
                                });
                            }
                        });

                    menu(ui, &a_chrome(), |_| {});
                });

            // The central panel, so the side panel is not the whole window.
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("the photograph");
            });
        };

        let input = |events: Vec<egui::Event>| egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        };

        // Twice, so the hit test the press is decided by has a frame of
        // rectangles behind it.
        for _ in 0..2 {
            let _ = ctx.run(input(Vec::new()), draw);
        }

        let moved = || vec![egui::Event::PointerMoved(at)];
        let _ = ctx.run(input(moved()), draw);

        let mut press = moved();
        press.push(egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Secondary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        });
        let _ = ctx.run(input(press), draw);

        // The popup is laid out on the frame after the one that opened it.
        let output = ctx.run(input(moved()), draw);

        painted(&output)
            .iter()
            .any(|text| text == "The history panel")
    }

    /// A menu that ends on a settings row that is not there ends on nothing.
    #[test]
    fn every_panel_ends_on_a_row_that_exists() {
        for chrome in EVERY_PANEL {
            let said = chrome.subject.said();
            assert!(
                crate::config::registry::row(chrome.setting).is_some(),
                "{said} ends on {}, which is not a row",
                chrome.setting
            );
        }
    }

    /// And a "bind a key" that arms a row that is not there arms nothing.
    #[test]
    fn every_key_a_panel_offers_to_bind_is_a_key() {
        for chrome in EVERY_PANEL {
            let Some(path) = chrome.key else {
                continue;
            };

            let said = chrome.subject.said();
            let row = crate::config::registry::row(path)
                .unwrap_or_else(|| panic!("{said} offers to bind {path}, which is not a row"));

            assert!(
                matches!(row.access, crate::config::registry::Access::Key(_, _)),
                "{said} offers to bind {path}, which is not a key"
            );
        }
    }

    /// A key that shows and hides a panel the menu cannot put away would be a
    /// row offering to bind something the menu never mentions.
    #[test]
    fn a_panel_that_offers_a_key_can_be_put_away() {
        for chrome in EVERY_PANEL {
            assert!(
                chrome.key.is_none() || chrome.hide.is_some(),
                "{} offers a key but no way to put it away",
                chrome.subject.said()
            );
        }
    }

    /// A menu says what it was asked about, so no two panels may say the same
    /// thing: the heading is the only thing telling them apart.
    #[test]
    fn no_two_panels_say_they_are_the_same_thing() {
        let mut said: Vec<String> = EVERY_PANEL
            .iter()
            .map(|chrome| chrome.subject.said())
            .collect();

        let all = said.len();
        said.sort();
        said.dedup();

        assert_eq!(said.len(), all, "two panels say the same thing: {said:?}");
    }

    /// Every piece of text the frame painted, in the order it was painted.
    fn painted(output: &egui::FullOutput) -> Vec<String> {
        output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) => Some(text.galley.text().to_string()),
                _ => None,
            })
            .collect()
    }
}

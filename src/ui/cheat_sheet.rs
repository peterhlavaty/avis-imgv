//! The keys, on one screen, on `?`.
//!
//! A viewer with sixty shortcuts and no way to see them is a viewer with about
//! six shortcuts, because those are the ones anybody remembers. The README has
//! them all, and the README is not on screen while somebody is culling.
//!
//! Generated from the binding registry rather than written out, for the reason
//! that matters more than the saved typing: it shows the keys that are
//! actually bound. Documentation drifts from a configuration the moment
//! somebody edits one, and a cheat sheet that lies about which key does what
//! is worse than none — this one reads the same table the key editor writes.
//!
//! Narrowed to where the user is, too. The image view's keys are no use while
//! looking at a contact sheet, and forty rows nobody can scan is the same
//! problem as no list at all.

use eframe::egui::{self, RichText};

use crate::app::mode::Mode;
use crate::config::{bindings, Config};

use super::keys::{describe, scopes_for};

/// One binding, ready to draw.
struct Row {
    key: String,
    name: &'static str,
    /// The sentence, which exists on every binding and was read only by the
    /// keyboard editor.
    description: &'static str,
    /// Its registry path, so a row can lead to the page that owns it.
    path: &'static str,
    /// Whether it can be changed at all.
    editable: bool,
}

/// What the mouse does here, as rows of the same shape as the keys.
///
/// On the sheet because a gesture nobody is told about is a gesture nobody
/// has: the wheel, the thumb buttons and the double click are all settings
/// now, and a person who has changed one has nowhere else to read back what
/// they changed it to. Every row leads to the control behind it.
fn mouse_rows(config: &Config, mode: Mode, needle: &str) -> Vec<Row> {
    let job = |wheel: crate::config::WheelJob| {
        crate::config::mouse::WHEEL_JOBS
            .iter()
            .find(|choice| choice.value == wheel.value())
            .map_or("Nothing", |choice| choice.label)
    };

    let verb = |name: &str| {
        crate::config::mouse::VERBS
            .iter()
            .find(|choice| choice.value == crate::config::mouse::verb_or_nothing(name))
            .map_or("Nothing", |choice| choice.label)
    };

    let sheet = mode == Mode::Grid;
    let mouse = &config.mouse;

    let rows: Vec<(String, &'static str, &'static str, &'static str)> = vec![
        (
            "Wheel".to_string(),
            if sheet {
                "Scroll the sheet"
            } else {
                job(mouse.wheel)
            },
            if sheet {
                "The sheet is an ordinary scrolling list, and wheel down is further in."
            } else {
                "One job at a time, and wheel down goes forward."
            },
            "mouse.wheel",
        ),
        (
            "Shift + wheel".to_string(),
            if sheet { "Ten rows" } else { "Ten photographs" },
            "The same step the page keys take.",
            "mouse.wheel",
        ),
        (
            "Ctrl + wheel".to_string(),
            if sheet {
                "Thumbnails per row"
            } else {
                job(mouse.ctrl_wheel)
            },
            "By convention: it is what a scrolling view does everywhere.",
            "mouse.ctrl_wheel",
        ),
        (
            "Alt + wheel".to_string(),
            if sheet {
                "Nothing"
            } else {
                "Move the photograph sideways"
            },
            "The axis the pan keys move on.",
            "mouse.ctrl_wheel",
        ),
        (
            "Click".to_string(),
            if sheet {
                "Pick this one out"
            } else {
                "Nothing; reserved"
            },
            "Ctrl adds one, Shift adds the run between.",
            "grid_view.click_opens",
        ),
        (
            "Double click".to_string(),
            if sheet {
                "Open it"
            } else {
                verb(&mouse.double_click)
            },
            "Never the only route to anything.",
            "mouse.double_click",
        ),
        (
            "Drag".to_string(),
            if sheet {
                "Pick out everything it crosses"
            } else {
                "Move the photograph"
            },
            "The two never share a surface, so they never share a button.",
            "mouse.drag",
        ),
        (
            "Middle drag".to_string(),
            if sheet {
                "Scroll the sheet"
            } else {
                "Move the photograph"
            },
            "Always, whether or not there is slack.",
            "mouse.drag",
        ),
        (
            "Middle click".to_string(),
            verb(&mouse.middle),
            "Nothing until you say otherwise: not every mouse has one.",
            "mouse.middle",
        ),
        (
            "Right click".to_string(),
            "The menu for what is under the pointer",
            "On the press, and Shift + F10 does the same from the keyboard.",
            "menus.settings_rows",
        ),
        (
            "Thumb back".to_string(),
            verb(&mouse.back),
            "On the down-stroke, with no double-click meaning.",
            "mouse.back",
        ),
        (
            "Thumb forward".to_string(),
            verb(&mouse.forward),
            "On the down-stroke, with no double-click meaning.",
            "mouse.forward",
        ),
        (
            "Drop a file".to_string(),
            "Open its folder, on that file",
            "A folder dropped on the window opens as itself.",
            "browsing.filter_follows_folder",
        ),
    ];

    rows.into_iter()
        .filter(|(key, name, description, _)| {
            needle.is_empty()
                || key.to_lowercase().contains(needle)
                || name.to_lowercase().contains(needle)
                || description.to_lowercase().contains(needle)
        })
        .map(|(key, name, description, path)| Row {
            key,
            name,
            description,
            path,
            editable: true,
        })
        .collect()
}

/// Draws the sheet, and reports whether it should stay open.
///
/// `just_opened` says this is the frame the key that opened it was pressed,
/// which is the frame that must not close it again — otherwise the sheet
/// appears and vanishes within one frame and nothing is ever seen.
///
/// It used to close on *any* key, which was right while it was a thing you
/// only glanced at and wrong the moment it had a search box: the first
/// character typed would have dismissed it. Escape, or any key while the box
/// does not hold the cursor. A click no longer closes it: as a window there was
/// an outside to click on, and as a card filling the window there is not — a
/// click anywhere in it would be a click on the sheet itself.
pub fn contents(
    ui: &mut egui::Ui,
    config: &Config,
    mode: Mode,
    just_opened: bool,
    query: &mut String,
    change: &mut Option<&'static str>,
) -> bool {
    let ctx = ui.ctx().clone();
    let ctx = &ctx;
    let mut open = true;
    let bindings = bindings::all();
    let needle = query.trim().to_lowercase();

    // Which keys are live where the user is. The sheet used to have to measure
    // its own height from this before it could be drawn — a window sized to a
    // scrolling area is sized to nothing, because a scrolling area is happy to
    // be one line tall. A card is the height of the window and has nothing to
    // work out.
    let live = scopes_for(mode);

    let mut sections: Vec<(&'static str, Vec<Row>)> = bindings::SECTIONS
        .iter()
        .map(|section| {
            let rows: Vec<Row> = bindings
                .iter()
                .filter(|binding| bindings::heading(binding) == *section)
                .filter(|binding| live.contains(&binding.scope()))
                .filter(|binding| binding.exists(config))
                .filter_map(|binding| {
                    // A fixed key has no shortcut field to read; its name is
                    // written on the row itself.
                    let key = match binding.fixed() {
                        Some(name) => name.to_string(),
                        None => describe(binding.get(config)?),
                    };

                    // Over the key as well as the name and the sentence: "what
                    // does F3 do" is asked as often as "what is the key for
                    // stacking".
                    if !needle.is_empty()
                        && !key.to_lowercase().contains(&needle)
                        && !binding.name().to_lowercase().contains(&needle)
                        && !binding.description().to_lowercase().contains(&needle)
                    {
                        return None;
                    }

                    Some(Row {
                        key,
                        name: binding.name(),
                        description: binding.description(),
                        path: binding.path(),
                        editable: binding.is_editable(),
                    })
                })
                .collect();

            (*section, rows)
        })
        .filter(|(_, rows)| !rows.is_empty())
        .collect();

    // Last, because the keys are what somebody came for; present, because a
    // gesture nobody is told about is a gesture nobody has.
    let pointer = mouse_rows(config, mode, &needle);
    if !pointer.is_empty() {
        sections.push(("The mouse", pointer));
    }

    let box_has_focus;

    {
        let field = ui.add(
            egui::TextEdit::singleline(query)
                .hint_text("Search the keys")
                .desired_width(f32::INFINITY),
        );
        box_has_focus = field.has_focus();

        egui::ScrollArea::vertical().show(ui, |ui| {
            if sections.is_empty() {
                ui.add_space(10.0);
                ui.weak(format!("No key here matches \"{}\".", query.trim()));
                return;
            }

            for (section, rows) in &sections {
                ui.add_space(6.0);
                ui.label(RichText::new(*section).heading());
                ui.add_space(4.0);

                egui::Grid::new(("cheat-sheet", section))
                    .num_columns(2)
                    .spacing([18.0, 4.0])
                    .show(ui, |ui| {
                        for row in rows {
                            // A route out, not only a statement. Every row
                            // opens the page that owns it, with its own
                            // binding armed.
                            let key = ui.add(
                                egui::Label::new(RichText::new(&row.key).monospace().strong())
                                    .sense(egui::Sense::click()),
                            );

                            if row.editable {
                                if key.on_hover_text("Click to change this").clicked() {
                                    *change = Some(row.path);
                                }
                            } else {
                                key.on_hover_text("The viewer reads this key itself");
                            }

                            // The sentence goes on the row rather than in a
                            // tooltip: this is the reference. It has been on
                            // every binding all along and was read only by
                            // the keyboard editor.
                            ui.vertical(|ui| {
                                ui.label(row.name);
                                ui.weak(RichText::new(row.description).small());
                            });
                            ui.end_row();
                        }
                    });
            }

            ui.add_space(10.0);
            ui.separator();

            // The footer was a statement; it is the route out. This sheet
            // is the best documentation in the program and it led nowhere.
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("These are the keys as configured. Escape closes this.").weak(),
                );

                if ui.button("Change them…").clicked() {
                    *change = Some("");
                }
            });
        });
    }

    if just_opened {
        return open;
    }

    // egui runs the frame again whenever something in it asks for another
    // look — a card whose size it does not yet know is one such — and the
    // second pass arrives with no events at all. Reading the keys there would
    // find none, and the answer of the pass that did read them would be thrown
    // away.
    if ctx.current_pass_index() > 0 {
        return open;
    }

    // Escape always, and any key while nobody is typing. Presses rather than
    // what is held, so a key still down from a moment ago does not close it
    // before it is read.
    let dismissed = ctx.input(|i| {
        let escaped = i.key_pressed(egui::Key::Escape);
        let any_key = i
            .events
            .iter()
            .any(|event| matches!(event, egui::Event::Key { pressed: true, .. }));

        escaped || (!box_has_focus && any_key)
    });

    if dismissed {
        open = false;
    }

    open
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One frame with the sheet in it, saying whether it stayed up.
    fn sheet(pressed: Option<egui::Key>, just_opened: bool, query: &str) -> bool {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            events: pressed
                .map(|key| egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                })
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let mut query = query.to_string();
        let mut change = None;
        let mut open = true;

        let _ = ctx.run(input, |ctx| {
            // As the program does it: a frame egui runs twice does not draw
            // the card again once the first pass has put it down.
            if !open {
                return;
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                open = contents(
                    ui,
                    &Config::default(),
                    Mode::Image,
                    just_opened,
                    &mut query,
                    &mut change,
                );
            });
        });

        open
    }

    /// The frame it opens on must not also close it, or nothing is ever seen.
    #[test]
    fn the_key_that_opens_it_does_not_also_close_it() {
        assert!(
            sheet(Some(egui::Key::F1), true, ""),
            "it closed on the frame it opened"
        );
    }

    /// And the next key does close it, while nobody is typing.
    #[test]
    fn the_next_key_closes_it() {
        assert!(!sheet(Some(egui::Key::A), false, ""));
    }

    /// A quiet frame leaves it up. A click is a quiet frame now: as a card
    /// filling the window there is no outside to click on.
    #[test]
    fn nothing_happening_leaves_it_open() {
        assert!(sheet(None, false, ""));
    }

    /// The sheet reads the configuration rather than a fixed list, so a
    /// rebound key shows as the user bound it.
    #[test]
    fn it_shows_the_key_the_user_bound() {
        let mut config = Config::default();
        config.general.sc_toggle_gallery = crate::config::Shortcut::new("j", &["ctrl"]);

        let bindings = bindings::all();
        let found = bindings
            .iter()
            .find(|binding| binding.name() == "Gallery")
            .and_then(|binding| binding.get(&config))
            .map(describe);

        assert_eq!(found, Some("Ctrl + j".to_string()));
    }

    /// A search that matches nothing says so rather than drawing an empty
    /// card, and the sheet stays up so the query can be corrected.
    #[test]
    fn a_search_that_matches_nothing_leaves_the_sheet_up() {
        assert!(sheet(None, false, "zzzznothing"));
    }
}

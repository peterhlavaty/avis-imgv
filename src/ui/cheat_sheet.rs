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

use super::keys::describe;

/// One binding, ready to draw.
struct Row {
    key: String,
    name: &'static str,
    /// The sentence, which exists on every binding and was read only by the
    /// keyboard editor.
    description: &'static str,
}

/// Which sections of the registry are worth showing in each mode.
///
/// "General" and "Ratings and tags" are everywhere, because they are: the
/// modes, the panels, the marks. The rest follows what is on screen.
fn sections_for(mode: Mode) -> &'static [&'static str] {
    match mode {
        Mode::Grid => &["General", "Gallery", "Ratings and tags"],
        Mode::Image | Mode::Slideshow => &["General", "Image view", "Ratings and tags"],
        // A folder job draws no photographs, so the marking and navigation
        // keys are not what somebody is looking for there.
        Mode::Rename | Mode::TimeShift | Mode::Group => &["General"],
    }
}

/// Draws the sheet, and reports whether it should stay open.
///
/// `just_opened` says this is the frame the key that opened it was pressed,
/// which is the frame that must not close it again — otherwise the sheet
/// appears and vanishes within one frame and nothing is ever seen.
///
/// It used to close on *any* key, which was right while it was a thing you
/// only glanced at and wrong the moment it had a search box: the first
/// character typed would have dismissed it. Escape, a click outside, or any key
/// while the box does not hold the cursor.
pub fn ui(
    ctx: &egui::Context,
    config: &Config,
    mode: Mode,
    just_opened: bool,
    query: &mut String,
) -> bool {
    let mut open = true;
    let bindings = bindings::all();
    let needle = query.trim().to_lowercase();

    // Gathered before anything is drawn, because the height has to be known
    // before the window is: a scrolling area has no natural height of its own
    // — it is happy to be one line tall — so a window sized to its contents
    // ends up sized to nothing, and the list is clipped after a dozen rows.
    let sections: Vec<(&'static str, Vec<Row>)> = sections_for(mode)
        .iter()
        .map(|section| {
            let rows: Vec<Row> = bindings
                .iter()
                .filter(|binding| binding.section == *section)
                .filter_map(|binding| {
                    let key = describe(binding.get(config)?);

                    // Over the key as well as the name and the sentence: "what
                    // does F3 do" is asked as often as "what is the key for
                    // stacking".
                    if !needle.is_empty()
                        && !key.to_lowercase().contains(&needle)
                        && !binding.name.to_lowercase().contains(&needle)
                        && !binding.description.to_lowercase().contains(&needle)
                    {
                        return None;
                    }

                    Some(Row {
                        key,
                        name: binding.name,
                        description: binding.description,
                    })
                })
                .collect();

            (*section, rows)
        })
        .filter(|(_, rows)| !rows.is_empty())
        .collect();

    let tallest = ctx.content_rect().height() * 0.75;
    let mut box_has_focus = false;

    egui::Window::new(format!("Keys — {}", mode.label()))
        .collapsible(false)
        .resizable(false)
        .default_width(620.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            let field = ui.add(
                egui::TextEdit::singleline(query)
                    .hint_text("Search the keys")
                    .desired_width(f32::INFINITY),
            );
            box_has_focus = field.has_focus();

            ui.set_height(needed_height(ui, &sections).min(tallest));

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
                                ui.label(RichText::new(&row.key).monospace().strong());

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
                ui.label(
                    RichText::new("These are the keys as configured. Escape closes this.").weak(),
                );
            });
        });

    if just_opened {
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

        escaped || (!box_has_focus && any_key) || i.pointer.any_click()
    });

    if dismissed {
        open = false;
    }

    open
}

/// How tall the sheet wants to be, from the fonts actually in use.
///
/// Measured rather than guessed at, so it is right whatever the configured
/// text scaling is: a sheet sized for the default font and drawn at 150% would
/// clip the last rows, which is precisely the failure it exists to avoid.
fn needed_height(ui: &egui::Ui, sections: &[(&str, Vec<Row>)]) -> f32 {
    let row = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
    let small = ui.text_style_height(&egui::TextStyle::Small) + 2.0;
    let heading = ui.text_style_height(&egui::TextStyle::Heading) + 10.0;

    let rows: usize = sections.iter().map(|(_, rows)| rows.len()).sum();
    let footer = row * 2.0 + 16.0;

    sections.len() as f32 * heading + rows as f32 * (row + small) + footer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_has_something_to_show() {
        for mode in Mode::ALL {
            assert!(!sections_for(*mode).is_empty(), "{mode:?}");
        }
    }

    /// The keys on screen are the ones for what is on screen.
    #[test]
    fn a_mode_shows_its_own_keys_and_not_the_others() {
        let grid = sections_for(Mode::Grid);
        let image = sections_for(Mode::Image);

        assert!(grid.contains(&"Gallery"));
        assert!(!grid.contains(&"Image view"));

        assert!(image.contains(&"Image view"));
        assert!(!image.contains(&"Gallery"));
    }

    /// Every section named here has to exist in the registry, or a rename
    /// would silently empty the sheet.
    #[test]
    fn every_named_section_is_a_real_one() {
        for mode in Mode::ALL {
            for section in sections_for(*mode) {
                assert!(
                    bindings::SECTIONS.contains(section),
                    "{section} is not a section of the registry"
                );
            }
        }
    }

    /// And every section of the registry is shown somewhere, or a whole group
    /// of keys would be undocumented on screen.
    #[test]
    fn every_section_of_the_registry_is_shown_in_some_mode() {
        for section in bindings::SECTIONS {
            assert!(
                Mode::ALL
                    .iter()
                    .any(|mode| sections_for(*mode).contains(section)),
                "{section} is shown in no mode"
            );
        }
    }

    /// The frame it opens on must not also close it, or nothing is ever seen.
    #[test]
    fn the_key_that_opens_it_does_not_also_close_it() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::F1,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        });

        let mut query = String::new();
        assert!(
            ui(&ctx, &Config::default(), Mode::Image, true, &mut query),
            "it closed on the frame it opened"
        );
    }

    /// And the next key does close it, while nobody is typing.
    #[test]
    fn the_next_key_closes_it() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        });

        let mut query = String::new();
        assert!(!ui(
            &ctx,
            &Config::default(),
            Mode::Image,
            false,
            &mut query
        ));
    }

    /// A quiet frame leaves it up.
    #[test]
    fn nothing_happening_leaves_it_open() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());

        let mut query = String::new();
        assert!(ui(&ctx, &Config::default(), Mode::Image, false, &mut query));
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
            .find(|binding| binding.name == "Gallery")
            .and_then(|binding| binding.get(&config))
            .map(describe);

        assert_eq!(found, Some("Ctrl + j".to_string()));
    }

    /// A search that matches nothing says so rather than drawing an empty
    /// window, and the sheet stays up so the query can be corrected.
    #[test]
    fn a_search_that_matches_nothing_leaves_the_sheet_up() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());

        let mut query = "zzzznothing".to_string();
        assert!(ui(&ctx, &Config::default(), Mode::Image, false, &mut query));
    }
}

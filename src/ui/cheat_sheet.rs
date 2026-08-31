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
/// Anything closes it: it is a thing you glance at, and having to find the
/// right key to dismiss the list of keys would be its own joke.
///
/// `just_opened` says this is the frame the key that opened it was pressed,
/// which is the frame that must not close it again — otherwise the sheet
/// appears and vanishes within one frame and nothing is ever seen.
pub fn ui(ctx: &egui::Context, config: &Config, mode: Mode, just_opened: bool) -> bool {
    let mut open = true;
    let bindings = bindings::all();

    // Gathered before anything is drawn, because the height has to be known
    // before the window is: a scrolling area has no natural height of its own —
    // it is happy to be one line tall — so a window sized to its contents ends
    // up sized to nothing, and the list is clipped after a dozen rows.
    let sections: Vec<(&'static str, Vec<(String, &'static str)>)> = sections_for(mode)
        .iter()
        .map(|section| {
            let rows = bindings
                .iter()
                .filter(|binding| binding.section == *section)
                .filter_map(|binding| Some((describe(binding.get(config)?), binding.name)))
                .collect();

            (*section, rows)
        })
        .filter(|(_, rows): &(_, Vec<_>)| !rows.is_empty())
        .collect();

    let tallest = ctx.content_rect().height() * 0.75;

    egui::Window::new(format!("Keys — {}", mode.label()))
        .collapsible(false)
        .resizable(false)
        .default_width(560.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_height(needed_height(ui, &sections).min(tallest));

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (section, rows) in &sections {
                    ui.add_space(6.0);
                    ui.label(RichText::new(*section).heading());
                    ui.add_space(4.0);

                    egui::Grid::new(("cheat-sheet", section))
                        .num_columns(2)
                        .spacing([18.0, 4.0])
                        .show(ui, |ui| {
                            for (key, name) in rows {
                                ui.label(RichText::new(key).monospace().strong());
                                ui.label(*name);
                                ui.end_row();
                            }
                        });
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label(
                    RichText::new("These are the keys as configured. Any key closes this.").weak(),
                );
            });
        });

    // Any key going down, and any click. Presses rather than what is held, so
    // a key still down from a moment ago does not close it before it is read.
    if just_opened {
        return open;
    }

    let pressed = ctx.input(|i| {
        i.events
            .iter()
            .any(|event| matches!(event, egui::Event::Key { pressed: true, .. }))
            || i.pointer.any_click()
    });

    if pressed {
        open = false;
    }

    open
}

/// How tall the sheet wants to be, from the fonts actually in use.
///
/// Measured rather than guessed at, so it is right whatever the configured
/// text scaling is: a sheet sized for the default font and drawn at 150% would
/// clip the last rows, which is precisely the failure it exists to avoid.
fn needed_height(ui: &egui::Ui, sections: &[(&str, Vec<(String, &str)>)]) -> f32 {
    let row = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
    let heading = ui.text_style_height(&egui::TextStyle::Heading) + 10.0;

    let rows: usize = sections.iter().map(|(_, rows)| rows.len()).sum();
    let footer = row * 2.0 + 16.0;

    sections.len() as f32 * heading + rows as f32 * row + footer
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

        assert!(
            ui(&ctx, &Config::default(), Mode::Image, true),
            "it closed on the frame it opened"
        );
    }

    /// And the next key does close it.
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

        assert!(!ui(&ctx, &Config::default(), Mode::Image, false));
    }

    /// A quiet frame leaves it up.
    #[test]
    fn nothing_happening_leaves_it_open() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());

        assert!(ui(&ctx, &Config::default(), Mode::Image, false));
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
}

//! The key a menu names beside its verb.
//!
//! A menu that offers something the keyboard also does names the key for it, on
//! the right of the row and in the weak colour — which is what every desktop
//! menu has done since menus had keys, and is the only route by which somebody
//! who opens a menu ever stops needing it. The key is rendered from the binding
//! rather than written into the label, so a rebind stays correct: Microsoft asks
//! exactly that of a menu that names a key, and the two rows in this program
//! that already named one had each done it their own way, one with brackets and
//! one with two spaces.
//!
//! It is published once a frame rather than threaded through the menus. Thirty
//! surfaces draw one, in a dozen files, and not one of them holds the
//! configuration — which is why `surface::more_settings` is a process-wide flag
//! and `panel::showing` a process-wide list. This is that shape again, with the
//! mode folded in: a key that is not read where the menu is drawn is a key the
//! menu must not name, and `Enter` opens the cell under the cursor in the
//! contact sheet and does nothing at all on the strip beside a photograph.

use std::sync::Mutex;

use eframe::egui::{self, Atom, Response, RichText};

use crate::app::mode::Mode;
use crate::config::registry::Scope;
use crate::config::{bindings, Config};

use super::describe_into;

/// One command, as everything that names a key sees it.
#[derive(Default)]
struct Named {
    /// Its path in the registry, which is its identity everywhere else.
    path: &'static str,
    /// What its keys read as. Empty where it has none, and where the ones it
    /// has are not read in the mode on screen.
    key: String,
}

/// One entry per binding, in the registry's order.
static NAMED: Mutex<Vec<Named>> = Mutex::new(Vec::new());

/// Which scopes are live in each mode.
///
/// Read off the registry rather than off a heading, which is the same change
/// the clash checker made: a scope states where a binding is *read*, and a
/// heading only happens to. `Everywhere` is in every mode, because it is.
///
/// Here rather than beside the cheat sheet that used to own it, because the
/// sheet and the menus ask one question — which keys are live where the person
/// is standing — and two answers to it would drift.
pub fn scopes_for(mode: Mode) -> &'static [Scope] {
    match mode {
        Mode::Grid => &[Scope::Everywhere, Scope::Gallery, Scope::Overlay],
        Mode::Image | Mode::Slideshow => &[Scope::Everywhere, Scope::ImageView, Scope::Overlay],
        // A folder job draws no photographs, so the marking and navigation
        // keys are not what somebody is looking for there.
        Mode::Rename | Mode::TimeShift | Mode::Group => &[Scope::Everywhere],
    }
}

/// Says what every command's keys read as, for whatever names one.
///
/// Written once a frame by `App::publish_keys`, beside the panels and for the
/// same reason. Read afresh rather than remembered: a copy kept beside the
/// settings would be right until somebody rebound a key by a route that forgot
/// to refresh it, and a menu naming a key that does nothing is worse than one
/// naming none. The strings are written over rather than built again, so a
/// frame on which nothing was rebound allocates nothing.
pub fn publish(config: &Config, mode: Mode) {
    let Ok(mut named) = NAMED.lock() else {
        return;
    };

    let bindings = bindings::all();
    let live = scopes_for(mode);

    named.resize_with(bindings.len(), Named::default);

    for (row, binding) in named.iter_mut().zip(bindings.iter()) {
        row.path = binding.path();
        row.key.clear();

        if !live.contains(&binding.scope()) {
            continue;
        }

        match binding.fixed() {
            // A key nobody can change is still a key the menu should name.
            Some(key) => row.key.push_str(key),
            None => {
                if let Some(shortcut) = binding.get(config).filter(|it| !it.is_empty()) {
                    describe_into(shortcut, &mut row.key);
                }
            }
        }
    }
}

/// What the keys for the command at `path` read as.
///
/// Empty where it has none and where the ones it has are not read here, which
/// are the two cases a menu draws the same way: nothing at all.
pub fn of(path: &str) -> String {
    // A surface with no key to name asks for none, rather than each of them
    // deciding for itself whether to ask: the panels are eight rows of which
    // one has no binding, and the row is otherwise the same row.
    if path.is_empty() {
        return String::new();
    }

    let Ok(named) = NAMED.lock() else {
        return String::new();
    };

    // A menu naming a path the registry has never heard of draws no key and
    // says nothing about it, which is how a renamed row would go quietly
    // wrong. Only worth asking once something has been published, since a test
    // that draws a menu without a frame around it asks an empty list.
    debug_assert!(
        named.is_empty() || named.iter().any(|row| row.path == path),
        "{path} is named by a menu and is not a key the registry has"
    );

    named
        .iter()
        .find(|row| row.path == path)
        .map(|row| row.key.clone())
        .unwrap_or_default()
}

/// A menu row that names its key: what it does on the left, the key on the
/// right.
///
/// The one way any of them is drawn, so that thirty surfaces cannot each choose
/// their own punctuation between the two halves. `Button::shortcut_text` is
/// egui's own answer for this: it appends a spring and the text after it, so
/// the key sits against the right edge of a row that a menu's justified layout
/// has already stretched to the width of the menu.
pub fn button(ui: &mut egui::Ui, label: impl Into<String>, path: &str) -> Response {
    let label = label.into();

    match of(path) {
        key if key.is_empty() => ui.button(label),
        key => ui.add(egui::Button::new(label).shortcut_text(key)),
    }
}

/// The same as one of a set, for a row that says which of them is in force.
pub fn radio(ui: &mut egui::Ui, chosen: bool, label: impl Into<String>, path: &str) -> Response {
    let label = label.into();

    match of(path) {
        key if key.is_empty() => ui.radio(chosen, label),
        key => ui.add(egui::RadioButton::new(
            chosen,
            (label, Atom::grow(), RichText::new(key).weak()),
        )),
    }
}

/// The same with a tick against it, for a row that says what is on screen.
///
/// `Checkbox` has no `shortcut_text` of its own, so the two atoms that method
/// appends are appended here instead: a spring to take up the slack, and the
/// key after it in the weak colour.
pub fn checkbox(
    ui: &mut egui::Ui,
    on: &mut bool,
    label: impl Into<String>,
    path: &str,
) -> Response {
    let label = label.into();

    match of(path) {
        key if key.is_empty() => ui.checkbox(on, label),
        key => ui.add(egui::Checkbox::new(
            on,
            (label, Atom::grow(), RichText::new(key).weak()),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table is process-wide, so the tests that write to it take turns
    /// rather than racing each other.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

    /// The turn, surviving a test that panicked while holding it.
    fn a_turn() -> std::sync::MutexGuard<'static, ()> {
        ONE_AT_A_TIME
            .lock()
            .unwrap_or_else(|held| held.into_inner())
    }

    /// What a fresh configuration binds, as the menus print it.
    #[test]
    fn a_command_is_named_by_the_key_it_carries() {
        let _turn = a_turn();
        publish(&Config::default(), Mode::Image);

        assert_eq!(of("general.sc_settings"), "Ctrl + Comma");
        assert_eq!(of("image_view.sc_fit"), "f");
    }

    /// A key nobody can change is still a key, and the menu still names it.
    #[test]
    fn a_fixed_key_is_named_too() {
        let _turn = a_turn();
        publish(&Config::default(), Mode::Image);

        assert_eq!(of("fixed.cheat_sheet"), "?");
    }

    /// The mode decides, because a key that is not read here does nothing
    /// here: `Enter` opens the cell under the cursor, and the contact sheet is
    /// the only place there is one.
    #[test]
    fn a_key_that_is_not_read_here_is_not_named() {
        let _turn = a_turn();

        publish(&Config::default(), Mode::Image);
        assert!(of("fixed.grid_open").is_empty());
        assert!(of("grid_view.sc_select_all").is_empty());

        publish(&Config::default(), Mode::Grid);
        assert_eq!(of("fixed.grid_open"), "Enter");
        assert!(!of("grid_view.sc_select_all").is_empty());

        // General is live in every one of them, including the three that draw
        // no photograph at all.
        publish(&Config::default(), Mode::Rename);
        assert!(!of("general.sc_settings").is_empty());
        assert!(of("image_view.sc_fit").is_empty());
    }

    /// A surface with no key at all asks for none, and is answered rather than
    /// complained about: the performance readout is a panel like the other
    /// seven and is put away by a key that is not a setting.
    #[test]
    fn a_surface_with_no_key_names_nothing() {
        let _turn = a_turn();
        publish(&Config::default(), Mode::Image);

        assert!(of("").is_empty());
    }

    /// A command somebody has taken the last key from names none, and draws
    /// the same as one that never had one.
    #[test]
    fn a_command_with_no_key_is_named_by_nothing() {
        let _turn = a_turn();

        let mut config = Config::default();
        config.image_view.sc_fit = crate::config::Shortcut::unbound();
        publish(&config, Mode::Image);

        assert!(of("image_view.sc_fit").is_empty());
    }

    #[test]
    fn every_mode_has_something_to_show() {
        for mode in Mode::ALL {
            assert!(!scopes_for(*mode).is_empty(), "{mode:?}");
        }
    }

    /// The keys on screen are the ones for what is on screen.
    #[test]
    fn a_mode_shows_its_own_keys_and_not_the_others() {
        let grid = scopes_for(Mode::Grid);
        let image = scopes_for(Mode::Image);

        assert!(grid.contains(&Scope::Gallery));
        assert!(!grid.contains(&Scope::ImageView));

        assert!(image.contains(&Scope::ImageView));
        assert!(!image.contains(&Scope::Gallery));
    }

    /// The keys read in every mode are shown in every mode, which is what
    /// makes the sheet a complete answer rather than most of one.
    #[test]
    fn what_is_read_everywhere_is_shown_everywhere() {
        for mode in Mode::ALL {
            assert!(scopes_for(*mode).contains(&Scope::Everywhere), "{mode:?}");
        }
    }

    /// Every scope a binding can carry is shown in some mode, or a whole group
    /// of keys would be undocumented on screen — and, now, a whole group of
    /// menu rows would name no key however they were bound.
    #[test]
    fn every_scope_a_binding_has_is_shown_in_some_mode() {
        for binding in bindings::all() {
            let scope = binding.scope();
            assert!(
                Mode::ALL
                    .iter()
                    .any(|mode| scopes_for(*mode).contains(&scope)),
                "{} is read in {} and shown in no mode",
                binding.path(),
                scope.label()
            );
        }
    }
}

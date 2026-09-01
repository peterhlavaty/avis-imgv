//! What the buttons on the mouse do.
//!
//! The three gestures with a single meaning — two clicks on the photograph,
//! the wheel pressed, and the two thumb buttons — hold the *name* of a command
//! rather than a meaning of their own. Every one of them is a second or third
//! route to something that already has a key, which is GNOME's rule: a gesture
//! that is physically demanding, "such as double-clicking or chording", must
//! never be the only way to anything. On a trackpad the middle button does not
//! exist at all.
//!
//! The resolution lives here rather than in either view because it is the one
//! place that knows both vocabularies: the application's commands and the
//! image view's. A view that had to know about the other would be a view that
//! knew about the whole program.

use eframe::egui::{self, PointerButton};

use crate::config::Config;
use crate::metadata::xmp::Flag;
use crate::view::image_view::input::Command as View;

use super::input::Command as App;
use super::{App as Application, Mode};

/// What a named command turns out to be.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Does {
    /// The name is not one this build knows, or is `nothing`.
    Nothing,
    /// Something the application does.
    App(App),
    /// Something the image view does.
    View(View),
}

/// The command a gesture's name stands for.
///
/// `config` is read for the zoom step, so that a wheel-less zoom gesture moves
/// by the same amount the zoom keys do rather than by a number written twice.
pub fn does(name: &str, config: &Config) -> Does {
    let step = if config.image_view.zoom_step > 1.0 {
        config.image_view.zoom_step
    } else {
        1.25
    };

    match name {
        "fit_or_actual" => Does::View(View::ToggleActualPixels),
        "fit" => Does::View(View::Fit),
        "fill" => Does::View(View::Fill),
        "actual_pixels" => Does::View(View::ZoomToPercent(100.0)),
        "zoom_in" => Does::View(View::ZoomBy(step)),
        "zoom_out" => Does::View(View::ZoomBy(1.0 / step)),
        "next" => Does::View(View::Next),
        "previous" => Does::View(View::Previous),
        "page_forward" => Does::View(View::PageForward),
        "page_back" => Does::View(View::PageBack),
        "first" => Does::View(View::First),
        "last" => Does::View(View::Last),
        "compare" => Does::View(View::Compare),
        "overlay" => Does::View(View::CycleOverlay),
        "marks" => Does::View(View::CycleMarks),
        "next_stack" => Does::App(App::NextStack),
        "previous_stack" => Does::App(App::PreviousStack),
        "fullscreen" => Does::App(App::ToggleFullscreen),
        "contact_sheet" => Does::App(App::ToggleGrid),
        "filmstrip" => Does::App(App::ToggleFilmstrip),
        "keywords" => Does::App(App::ToggleTagPanel),
        "information" => Does::App(App::ToggleSidePanel),
        "filter" => Does::App(App::ToggleFilter),
        "turn_left" => Does::App(App::Turn(false)),
        "turn_right" => Does::App(App::Turn(true)),
        "keep" => Does::App(App::SetFlag(Flag::Picked)),
        "reject" => Does::App(App::SetFlag(Flag::Rejected)),
        "move_to" => Does::App(App::MoveTo),
        "copy_to" => Does::App(App::CopyTo),
        "to_rejected_folder" => Does::App(App::ToRejectedFolder),
        "delete" => Does::App(App::Delete),
        "undo" => Does::App(App::Undo),
        "keys" => Does::App(App::ShowKeys),
        "settings" => Does::App(App::ShowSettings),
        "exit" => Does::App(App::Exit),
        _ => Does::Nothing,
    }
}

/// Where else each gesture verb lives.
///
/// A gesture must never be the only route to anything. GNOME's pointer
/// guidance says actions that are physically demanding, "such as
/// double-clicking or chording", should be avoided; on a trackpad the
/// secondary click is a two-finger tap and the middle button does not exist at
/// all. So every verb a gesture can be bound to has a key as well, and the
/// test below says so — which is what stops the next gesture from being a
/// command with one home.
///
/// The paths are the registry's. Where a verb is a toggle between two things
/// that have a key each, both are named: two clicks moving between fitted and
/// actual pixels is reachable as `image_view.sc_fit` and
/// `image_view.sc_one_to_one`, one press each.
#[cfg(test)]
const HOMES: &[(&str, &[&str])] = &[
    ("nothing", &[]),
    (
        "fit_or_actual",
        &["image_view.sc_fit", "image_view.sc_one_to_one"],
    ),
    ("fit", &["image_view.sc_fit"]),
    ("fill", &["image_view.sc_fit_maximize"]),
    ("actual_pixels", &["image_view.sc_one_to_one"]),
    ("zoom_in", &["image_view.sc_zoom_in"]),
    ("zoom_out", &["image_view.sc_zoom_out"]),
    ("next", &["image_view.sc_next"]),
    ("previous", &["image_view.sc_prev"]),
    ("next_stack", &["general.sc_next_stack"]),
    ("previous_stack", &["general.sc_previous_stack"]),
    ("page_forward", &["fixed.page_forward"]),
    ("page_back", &["fixed.page_back"]),
    ("first", &["fixed.first"]),
    ("last", &["fixed.last"]),
    ("fullscreen", &["general.sc_fullscreen"]),
    ("contact_sheet", &["general.sc_toggle_gallery"]),
    ("filmstrip", &["general.sc_filmstrip"]),
    ("keywords", &["tags.sc_toggle_tag_panel"]),
    ("information", &["general.sc_toggle_side_panel"]),
    ("filter", &["general.sc_filter"]),
    ("compare", &["image_view.sc_compare"]),
    ("overlay", &["image_view.sc_overlay"]),
    ("marks", &["image_view.sc_marks"]),
    ("turn_left", &["general.sc_turn_left"]),
    ("turn_right", &["general.sc_turn_right"]),
    ("keep", &["tags.sc_pick"]),
    ("reject", &["tags.sc_reject"]),
    ("move_to", &["cull.sc_move"]),
    ("copy_to", &["cull.sc_copy"]),
    ("to_rejected_folder", &["cull.sc_reject_folder"]),
    ("delete", &["general.sc_delete"]),
    ("undo", &["history.sc_undo"]),
    ("keys", &["fixed.cheat_sheet"]),
    ("settings", &["general.sc_settings"]),
    ("exit", &["general.sc_exit"]),
];

impl Application {
    /// Reads the buttons and runs whatever they are bound to.
    ///
    /// Nothing here fires while a text field has the keyboard or while the
    /// pointer is over a panel, a window or a menu: those surfaces have their
    /// own middle-click and double-click meanings, and a gesture that reached
    /// past them would act on a photograph the pointer is nowhere near.
    pub(super) fn handle_gestures(&mut self, ctx: &egui::Context) {
        if crate::utils::are_inputs_muted(ctx) || ctx.is_pointer_over_area() {
            return;
        }

        let mouse = self.settings.mouse.clone();

        // Two clicks on the photograph. Not in the contact sheet, where a
        // double click opens the cell under it and says so.
        if self.mode != Mode::Grid
            && ctx.input(|i| i.pointer.button_double_clicked(PointerButton::Primary))
        {
            self.run_gesture(&mouse.double_click);
        }

        if ctx.input(|i| i.pointer.button_clicked(PointerButton::Middle)) {
            self.run_gesture(&mouse.middle);
        }

        // On the down-stroke, and with no double-click meaning ever: a viewer
        // that waits to see whether a side-button click is a double makes
        // walking a folder feel slow and still moves one frame.
        for (button, verb) in [
            (PointerButton::Extra1, &mouse.back),
            (PointerButton::Extra2, &mouse.forward),
        ] {
            if ctx.input(|i| i.pointer.button_pressed(button)) {
                self.run_gesture(verb);
            }
        }
    }

    /// Runs the command of that name, wherever it lives.
    fn run_gesture(&mut self, name: &str) {
        match does(name, &self.settings) {
            Does::Nothing => {}
            Does::App(command) => self.apply_command(command),
            Does::View(command) => self.image_view.queue(command),
        }
    }
}

/// Whether `paths` is worth opening, and what to open it on.
///
/// A folder opens as itself; a file opens its folder and lands on it, which is
/// what double-clicking a photograph in a file manager means. Several files
/// dropped together are one folder and the first of them, because the viewer
/// shows a folder rather than a list.
pub fn dropped(
    paths: &[std::path::PathBuf],
) -> Option<(std::path::PathBuf, Option<std::path::PathBuf>)> {
    let first = paths.first()?;

    if first.is_dir() {
        return Some((first.clone(), None));
    }

    let parent = first.parent()?.to_path_buf();
    Some((parent, Some(first.clone())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mouse::VERBS;

    /// Every name the control offers reaches a command. A list that offers a
    /// verb the program cannot carry out is a setting that silently does
    /// nothing, which is the whole fault this stage is about.
    #[test]
    fn every_verb_in_the_list_does_something() {
        let config = Config::default();

        for verb in VERBS {
            let found = does(verb.value, &config);

            if verb.value == "nothing" {
                assert_eq!(found, Does::Nothing);
                continue;
            }

            assert_ne!(found, Does::Nothing, "{} does nothing", verb.value);
        }
    }

    /// And a name from a hand-edited file that this build has never heard of
    /// does nothing rather than something else.
    #[test]
    fn an_unknown_name_does_nothing() {
        assert_eq!(does("teleport", &Config::default()), Does::Nothing);
    }

    /// The shipped defaults, end to end.
    #[test]
    fn the_defaults_reach_the_commands_they_name() {
        let config = Config::default();

        assert_eq!(
            does(&config.mouse.double_click, &config),
            Does::View(View::ToggleActualPixels)
        );
        assert_eq!(does(&config.mouse.middle, &config), Does::Nothing);
        assert_eq!(
            does(&config.mouse.back, &config),
            Does::View(View::Previous)
        );
        assert_eq!(does(&config.mouse.forward, &config), Does::View(View::Next));
    }

    /// No command has fewer than two homes.
    ///
    /// Every verb a gesture can be bound to also has a key, so the gesture is
    /// a second route rather than the only one. This is four lines and it
    /// prevents the whole class of command that exists with no route anybody
    /// will find.
    #[test]
    fn every_gesture_is_a_second_route_and_not_the_only_one() {
        let paths: Vec<&str> = crate::config::registry::rows()
            .iter()
            .map(|row| row.path)
            .collect();

        for verb in VERBS {
            let (_, homes) = HOMES
                .iter()
                .find(|(name, _)| *name == verb.value)
                .unwrap_or_else(|| panic!("{} has no home listed", verb.value));

            if verb.value == "nothing" {
                continue;
            }

            assert!(!homes.is_empty(), "{} is only a gesture", verb.value);

            for home in *homes {
                assert!(
                    paths.contains(home),
                    "{} claims a home at {home}, which is not a row",
                    verb.value
                );
                assert!(
                    crate::config::bindings::is_a_key(home),
                    "{home} is a row but not a key"
                );
            }
        }
    }

    /// And nothing is listed as a home that is not a verb, so the table cannot
    /// rot quietly in the other direction either.
    #[test]
    fn nothing_is_listed_that_is_not_a_verb() {
        for (name, _) in HOMES {
            assert!(
                VERBS.iter().any(|verb| verb.value == *name),
                "{name} is not a verb"
            );
        }
    }

    /// A folder dropped on the window opens as itself.
    #[test]
    fn a_dropped_folder_opens_as_a_folder() {
        let folder = std::env::temp_dir();
        let found = dropped(std::slice::from_ref(&folder)).expect("something to open");

        assert_eq!(found, (folder, None));
    }

    /// A file opens its folder and lands on it, which is what double-clicking
    /// a photograph in a file manager means.
    #[test]
    fn a_dropped_file_opens_its_folder_on_it() {
        let file = std::env::temp_dir().join("DSCF0001.RAF");
        let (folder, land_on) = dropped(std::slice::from_ref(&file)).expect("something to open");

        assert_eq!(folder, std::env::temp_dir());
        assert_eq!(land_on, Some(file));
    }

    /// Several at once are one folder: the viewer shows a folder, not a list.
    #[test]
    fn several_dropped_files_are_one_folder() {
        let first = std::env::temp_dir().join("a.jpg");
        let second = std::env::temp_dir().join("b.jpg");

        let (_, land_on) = dropped(&[first.clone(), second]).expect("something to open");
        assert_eq!(land_on, Some(first));
    }

    /// A drop of nothing opens nothing, rather than the working directory.
    #[test]
    fn dropping_nothing_opens_nothing() {
        assert_eq!(dropped(&[]), None);
    }

    /// A gesture that zooms moves by the same amount the zoom keys do.
    #[test]
    fn a_zoom_gesture_takes_the_configured_step() {
        let mut config = Config::default();
        config.image_view.zoom_step = 2.0;

        assert_eq!(does("zoom_in", &config), Does::View(View::ZoomBy(2.0)));
        assert_eq!(does("zoom_out", &config), Does::View(View::ZoomBy(0.5)));
    }
}

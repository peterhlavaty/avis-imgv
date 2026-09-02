//! The status bar under the image: position, name, zoom.

use eframe::egui::{self, Sense};
use eframe::epaint::Vec2;

use crate::decoder::overlays::Overlay;
use crate::metadata::xmp::{leaf_of, Flag, Label, Xmp};
use crate::organize::pairs::Prefer;
use crate::view::image_view::opening::Opening;
use crate::view::stacks::Place;

use super::input::{Anchor, Command};

/// Zoom levels offered in the magnification context menu.
const PERCENTAGES: &[f32] = &[200., 100., 75., 50., 25.];

/// What the user has said about the photograph on screen.
///
/// Drawn in the bar so that rating, flagging or labelling with the panel shut
/// is not a keystroke that appears to do nothing.
#[derive(Debug, Clone, Default)]
pub struct Marks {
    pub stars: u8,
    pub flag: Flag,
    pub label: Option<Label>,
    /// Kept here as well as in the annotation store, because the filter asks
    /// about every photograph in the folder at once and a lookup per file per
    /// keystroke is the thing this list exists to avoid.
    ///
    /// With their levels where the sidecar records them, so narrowing by
    /// `Slovakia` finds everything filed underneath it and not only what is
    /// tagged with the word itself.
    pub keywords: Vec<String>,
}

impl Marks {
    pub fn of(annotations: &Xmp) -> Marks {
        Marks {
            stars: annotations.stars(),
            flag: annotations.flag(),
            label: annotations.known_label(),
            keywords: annotations
                .keywords
                .iter()
                .map(|keyword| {
                    annotations
                        .hierarchy
                        .iter()
                        .find(|path| leaf_of(path) == keyword)
                        .unwrap_or(keyword)
                        .clone()
                })
                .collect(),
        }
    }
}

/// Modes worth telling the user about.
#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    pub flattened: bool,
    pub watching: bool,
    /// What a photograph is drawn at on the frame it first appears.
    pub opening: crate::view::image_view::opening::Opening,
    /// Whether a mark moves on to the next photograph by itself.
    pub advancing: bool,
    /// Whether a set of photographs is pinned side by side.
    pub comparing: bool,
    /// Whether this photograph is a raw and a JPEG shot together.
    ///
    /// Said out loud because everything that follows — a rating, a move, a
    /// deletion — is about to happen to two files, and somebody who has
    /// forgotten the camera was set that way should not find that out
    /// afterwards.
    pub paired: bool,
    /// Which frame of which run this photograph is, when the folder is being
    /// shown stacked.
    ///
    /// The one thing a stacked folder has to keep saying: a cell that stands
    /// for seventeen frames looks exactly like a cell that stands for one, and
    /// somebody culling has to know which they are looking at.
    pub place: Option<Place>,
    /// Which mask is painted over the photograph, when one is.
    ///
    /// `Overlay::label` has existed since the overlays were written and was
    /// called by a test alone; the marking mode reached no status flag at all,
    /// so a photograph covered in red was unexplained.
    pub marking: Overlay,
}

/// Everything the bar draws, borrowed from the view.
pub struct Status<'a> {
    pub jump_to: &'a mut String,
    /// Whether a key has just asked for the box to take the keyboard.
    pub asking_to_go_to: bool,
    /// One based, as shown to the user.
    pub position: usize,
    pub total: usize,
    /// How many the filter is holding back, so a shorter collection is not a
    /// mystery.
    pub hidden: usize,
    pub name: String,
    pub percentage_zoom: f32,
    /// The smallest magnification the zoom will reach, or nought when nothing
    /// holds it.
    ///
    /// The rail ends where the zoom does. A stretch of rail that asks for
    /// something the view will refuse is worse on a fine drag than it sounds:
    /// the drag would carry on past the end and have to be walked all the way
    /// back before the handle moved again.
    pub least_zoom: f32,
    pub marks: Marks,
    pub flags: Flags,
    /// Which mode is on screen.
    ///
    /// `Mode::label()` is drawn in three places, none of them where people
    /// spend their time, while one key cycles all six and three of the six draw
    /// no photographs at all.
    pub mode: crate::app::mode::Mode,
    /// How many messages have arrived and not been read.
    ///
    /// One of the two ways into the history, so it is reachable without the
    /// menu bar being up.
    pub unread: usize,
}

/// What the user asked for by clicking in the bar.
#[derive(Debug, Default)]
pub struct Outcome {
    pub commands: Vec<Command>,
    /// A zero based index typed into the jump field.
    pub jump_to: Option<usize>,
    /// What the bar's own words asked for that the view cannot do itself.
    pub bar: Vec<BarAction>,
}

/// What one of the status bar's words was clicked to do.
///
/// Six words used to sit in the bar as bare labels with no tooltip and no way
/// to act on them, two of them the only place in the running program a setting
/// was visible at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarAction {
    /// Fold sub-directories into the collection, or stop.
    ToggleFlatten,
    /// Watch the folder for changes, or stop.
    ToggleWatching,
    /// Whether a mark moves on to the next photograph by itself.
    SetAdvancing(bool),
    /// What a photograph is drawn at when it comes up.
    SetOpening(crate::view::image_view::opening::Opening),
    /// Which half of a raw+JPEG pair is the one browsed.
    SetPairing(Prefer),
    /// Go to the settings row behind this readout.
    ///
    /// The reverse trip: somebody who opened the window out of habit learns the
    /// shorter route, and somebody who found the short route can still get to
    /// the page.
    Settings(&'static str),
    /// Arm the keyboard editor on the row that binds this.
    BindKey(&'static str),
    /// Switch to this mode.
    Mode(crate::app::mode::Mode),
    /// Open or fold the run this frame belongs to.
    ToggleStack,
    /// Narrow the folder to the photographs carrying this mark.
    ///
    /// The one verb that closes every dead end of its kind: a mark drawn on
    /// screen and no way to say "show me those".
    ShowOnlyFlag(Flag),
    ShowOnlyLabel(Label),
    ShowOnlyStars(u8),
    /// Set the narrowing rules aside without forgetting them.
    ShowEverything,
    /// Open the history of what the viewer has said.
    ShowMessages,
}

/// One word at the left end saying which mode is on screen.
///
/// There was no mode indicator at all: `Mode::label()` is drawn in the menu, in
/// the cheat sheet's title and in the organiser's own heading, and `F2` cycles
/// six modes of which three draw no photographs. Somebody who pressed it once
/// too often had nothing on screen telling them where they were.
fn mode_word(ui: &mut egui::Ui, mode: crate::app::mode::Mode) -> Vec<BarAction> {
    use crate::app::mode::Mode;

    let mut asked = Vec::new();

    let word =
        ui.add(egui::Label::new(egui::RichText::new(mode.label()).strong()).sense(Sense::click()));

    crate::ui::surface::with_menu(
        ui,
        &word,
        crate::ui::surface::Subject::of("Mode", mode.label()),
        "Which mode the window is in.",
        |ui| {
            for wanted in Mode::ALL {
                // Radios rather than buttons: the menu is also where somebody finds
                // out which mode they are in, which is the whole point of the word.
                if ui.radio(mode == *wanted, wanted.label()).clicked() {
                    asked.push(BarAction::Mode(*wanted));
                    ui.close();
                }
            }

            if crate::ui::surface::more_settings(ui, crate::config::registry::Page::OpeningAFolder)
            {
                asked.push(BarAction::Settings("general.start_in"));
                ui.close();
            }
        },
    );

    asked
}

/// Draws the words saying what mode the viewer is in, and takes their clicks.
///
/// Each word is a door: it says what it means, and its menu carries the verb
/// that turns it off. Four of the six carry a verb the program already has;
/// **Advancing** and **RAW+JPEG** carry a setting, and are the only place in
/// the running program either of those two is visible.
fn flag_words(ui: &mut egui::Ui, flags: &Flags, commands: &mut Vec<Command>) -> Vec<BarAction> {
    use crate::config::registry::Page;
    use crate::ui::surface;

    let mut asked = Vec::new();

    if flags.flattened {
        word(
            ui,
            "Flattened",
            "Photographs in sub-folders are part of this collection.",
            |ui| {
                if ui.button("Only this folder").clicked() {
                    asked.push(BarAction::ToggleFlatten);
                    ui.close();
                }
                if surface::bind_a_key(ui, "flattening") {
                    asked.push(BarAction::BindKey("general.sc_flatten_dir"));
                    ui.close();
                }
                if surface::more_settings(ui, Page::OpeningAFolder) {
                    asked.push(BarAction::Settings("browsing.sort"));
                    ui.close();
                }
            },
        );
    }

    if flags.watching {
        word(
            ui,
            "Watching",
            "The folder is being watched, so a file written into it appears.",
            |ui| {
                if ui.button("Stop watching the folder").clicked() {
                    asked.push(BarAction::ToggleWatching);
                    ui.close();
                }
                if surface::bind_a_key(ui, "watching") {
                    asked.push(BarAction::BindKey("general.sc_watch_directory"));
                    ui.close();
                }
                if surface::more_settings(ui, Page::OpeningAFolder) {
                    asked.push(BarAction::Settings("browsing.filter_follows_folder"));
                    ui.close();
                }
            },
        );
    }

    if let Some(said) = flags.opening.word() {
        word(
            ui,
            said,
            "What every photograph is drawn at when it comes up. Fitted says \
             nothing here, being what a viewer usually does.",
            |ui| {
                for wanted in Opening::ALL {
                    // Radios rather than buttons: the word says which of the
                    // three is in force, and the menu is where the other two
                    // are, so it may as well say it twice.
                    if ui.radio(flags.opening == *wanted, wanted.label()).clicked() {
                        asked.push(BarAction::SetOpening(*wanted));
                        ui.close();
                    }
                }
                if surface::bind_a_key(ui, "how a photograph opens") {
                    asked.push(BarAction::BindKey("image_view.sc_cycle_opening"));
                    ui.close();
                }
                if surface::more_settings(ui, Page::ThePhotograph) {
                    asked.push(BarAction::Settings("image_view.opening"));
                    ui.close();
                }
            },
        );
    }

    if flags.advancing {
        word(
            ui,
            "Advancing",
            "A star, a flag or a label moves on to the next photograph.",
            |ui| {
                if ui
                    .button("Stay on the photograph after marking it")
                    .clicked()
                {
                    asked.push(BarAction::SetAdvancing(false));
                    ui.close();
                }
                if surface::bind_a_key(ui, "advancing") {
                    asked.push(BarAction::BindKey("tags.sc_toggle_advance"));
                    ui.close();
                }
                if surface::more_settings(ui, Page::Marks) {
                    asked.push(BarAction::Settings("tags.advance_after_marking"));
                    ui.close();
                }
            },
        );
    }

    if flags.comparing {
        word(
            ui,
            "Comparing",
            "Several photographs are pinned side by side.",
            |ui| {
                if ui.button("Stop comparing").clicked() {
                    commands.push(Command::StopComparing);
                    ui.close();
                }
                if surface::more_settings(ui, Page::ThePhotograph) {
                    asked.push(BarAction::Settings("image_view.nr_images_shown"));
                    ui.close();
                }
            },
        );
    }

    if flags.marking != Overlay::Off {
        word(
            ui,
            flags.marking.label(),
            "A mask is painted over the photograph. Help → What the marks mean says \
             what the colours are.",
            |ui| {
                if ui.button("Show the photograph as it is").clicked() {
                    commands.push(Command::CycleMarks);
                    ui.close();
                }
                if surface::bind_a_key(ui, "the mask") {
                    asked.push(BarAction::BindKey("image_view.sc_marks"));
                    ui.close();
                }
            },
        );
    }

    if flags.paired {
        // The three sentences this needs were written when pairing was built
        // and drawn nowhere; this is the only place in the running program the
        // setting behind them is visible.
        word(
            ui,
            "RAW+JPEG",
            "This frame was shot as two files, and a rating, a move or a deletion is \
             about to happen to both.",
            |ui| {
                for prefer in Prefer::ALL {
                    if ui.button(prefer.label()).clicked() {
                        asked.push(BarAction::SetPairing(*prefer));
                        ui.close();
                    }
                }
                if surface::more_settings(ui, Page::RawFiles) {
                    asked.push(BarAction::Settings("raw.pair_with_jpeg"));
                    ui.close();
                }
            },
        );
    }

    asked
}

/// One clickable word in the flags row, and the menu it carries.
///
/// The word is its own subject: it is the whole of what was clicked, and six
/// of them stand in a row saying one word each, so the menu opens by saying
/// which of the six it belongs to.
fn word<R>(
    ui: &mut egui::Ui,
    text: &str,
    hint: &str,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    use crate::ui::surface;

    let word = ui.add(egui::Label::new(text).sense(Sense::click()));

    surface::with_menu(ui, &word, surface::Subject::the(text), hint, contents)
}

/// Draws the bar and reports the interactions it produced.
pub fn ui(ctx: &egui::Context, status: &mut Status<'_>) -> Outcome {
    let mut outcome = Outcome::default();

    egui::TopBottomPanel::bottom("image_view_bottom_bar")
        .show_separator_line(false)
        .show(ctx, |ui| {
            // Readable but not pressable while a window is in front: every
            // button on it changes what the photograph behind is doing.
            if crate::utils::is_a_window_in_front(ui.ctx()) {
                ui.disable();
            }

            ui.horizontal_centered(|ui| {
                outcome.bar.extend(mode_word(ui, status.mode));

                outcome.jump_to = jump_field(ui, status);

                let counted = match status.hidden {
                    0 => format!("{}/{}", status.position, status.total),
                    hidden => format!("{}/{} (+{hidden})", status.position, status.total),
                };

                let counter = ui.add_sized(
                    Vec2::new(
                        if status.hidden == 0 { 45. } else { 90. },
                        ui.available_height(),
                    ),
                    egui::Label::new(counted.as_str()).sense(Sense::click()),
                );

                // Two calls rather than one with an empty string: an empty
                // hover text still lays out and paints a frame.
                let says = match status.hidden {
                    0 => "Which photograph of how many, in the order on screen.".to_string(),
                    hidden => format!(
                        "Which photograph of how many. {hidden} more are hidden by the filter."
                    ),
                };

                crate::ui::surface::with_menu(
                    ui,
                    &counter,
                    crate::ui::surface::Subject::of("Position", &counted),
                    &says,
                    |ui| {
                        if status.hidden > 0 && ui.button("Show everything").clicked() {
                            outcome.bar.push(BarAction::ShowEverything);
                            ui.close();
                        }

                        if crate::ui::surface::more_settings(
                            ui,
                            crate::config::registry::Page::OpeningAFolder,
                        ) {
                            outcome.bar.push(BarAction::Settings("browsing.flag"));
                            ui.close();
                        }
                    },
                );

                if let Some(place) = status.flags.place {
                    let colour = if place.collapsed {
                        egui::Color32::from_rgb(226, 186, 120)
                    } else {
                        ui.visuals().text_color()
                    };

                    let said = ui.add(
                        egui::Label::new(egui::RichText::new(place.describe()).color(colour))
                            .sense(Sense::click()),
                    );

                    let run = place.describe();

                    crate::ui::surface::with_menu(
                        ui,
                        &said,
                        crate::ui::surface::Subject::of("Run", &run),
                        "One run of frames. Amber says it is folded up.",
                        |ui| {
                            if ui
                                .button(if place.collapsed {
                                    "Open this run"
                                } else {
                                    "Fold it back up"
                                })
                                .clicked()
                            {
                                outcome.bar.push(BarAction::ToggleStack);
                                ui.close();
                            }

                            if crate::ui::surface::bind_a_key(ui, "opening a run") {
                                outcome
                                    .bar
                                    .push(BarAction::BindKey("general.sc_toggle_stack"));
                                ui.close();
                            }

                            if crate::ui::surface::more_settings(
                                ui,
                                crate::config::registry::Page::OpeningAFolder,
                            ) {
                                outcome.bar.push(BarAction::Settings("group.max_gap"));
                                ui.close();
                            }
                        },
                    );
                }

                outcome
                    .bar
                    .extend(flag_words(ui, &status.flags, &mut outcome.commands));

                outcome.bar.extend(marks(ui, &status.marks));

                // Leave room for the zoom controls pinned to the right.
                let name_width = (ui.available_width() - 245.).max(20.);
                let name = ui.add_sized(
                    Vec2::new(name_width, ui.available_height()),
                    egui::Label::new(status.name.clone())
                        .truncate()
                        .sense(Sense::click()),
                );

                crate::ui::surface::with_menu(
                    ui,
                    &name,
                    crate::ui::surface::Subject::of("Photograph", &status.name),
                    "What this photograph is called, in the template you set.",
                    |ui| {
                        if ui.button("Copy the name").clicked() {
                            ui.ctx().copy_text(status.name.clone());
                            ui.close();
                        }

                        if crate::ui::surface::more_settings(
                            ui,
                            crate::config::registry::Page::ThePhotograph,
                        ) {
                            outcome
                                .bar
                                .push(BarAction::Settings("image_view.name_format"));
                            ui.close();
                        }
                    },
                );

                ui.with_layout(
                    egui::Layout::right_to_left(eframe::emath::Align::Max),
                    |ui| {
                        if status.unread > 0 {
                            let count = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(format!("✉ {}", status.unread)).small(),
                                )
                                .sense(Sense::click()),
                            );

                            if count
                                .on_hover_text(
                                    "Messages you have not read. The band holds four for \
                                     six seconds; this does not.",
                                )
                                .clicked()
                            {
                                outcome.bar.push(BarAction::ShowMessages);
                            }
                        }

                        outcome.commands.extend(zoom_slider(
                            ui,
                            status.percentage_zoom,
                            status.least_zoom,
                        ));

                        outcome
                            .commands
                            .extend(zoom_label(ui, status.percentage_zoom));
                    },
                );
            });
        });

    outcome
}

/// The three marks, drawn only when there is something to draw: the bar is a
/// summary, not a control, and an unmarked photograph should say nothing.
///
/// Each of them is a door now. A true statement that cannot be acted on is the
/// commonest kind of dead end in this program, and "three stars" is one of the
/// most obviously actionable things on screen.
fn marks(ui: &mut egui::Ui, marks: &Marks) -> Vec<BarAction> {
    use crate::config::registry::Page;
    use crate::ui::surface;

    let mut asked = Vec::new();

    if marks.flag != Flag::Unflagged {
        let colour = match marks.flag {
            Flag::Rejected => egui::Color32::from_rgb(219, 96, 96),
            _ => ui.visuals().text_color(),
        };

        let glyph = ui.add(
            egui::Label::new(egui::RichText::new(marks.flag.glyph()).color(colour))
                .sense(Sense::click()),
        );

        surface::with_menu(
            ui,
            &glyph,
            surface::Subject::of("Flag", marks.flag.name()),
            "The flag on this photograph.",
            |ui| {
                if ui.button("Show only these").clicked() {
                    asked.push(BarAction::ShowOnlyFlag(marks.flag));
                    ui.close();
                }
                if surface::more_settings(ui, Page::Marks) {
                    asked.push(BarAction::Settings("tags.advance_after_marking"));
                    ui.close();
                }
            },
        );
    }

    if let Some(label) = marks.label {
        let (r, g, b) = label.colour();
        let swatch = ui.add(
            egui::Label::new(egui::RichText::new("■").color(egui::Color32::from_rgb(r, g, b)))
                .sense(Sense::click()),
        );

        surface::with_menu(
            ui,
            &swatch,
            surface::Subject::of("Colour", label.name()),
            label.name(),
            |ui| {
                if ui.button("Show only these").clicked() {
                    asked.push(BarAction::ShowOnlyLabel(label));
                    ui.close();
                }
                if surface::bind_a_key(ui, "this colour") {
                    asked.push(BarAction::BindKey("tags.sc_label[0]"));
                    ui.close();
                }
                if surface::more_settings(ui, Page::Marks) {
                    asked.push(BarAction::Settings("tags.sc_label"));
                    ui.close();
                }
            },
        );
    }

    if marks.stars > 0 {
        let shown = ui.add(egui::Label::new(stars(marks.stars)).sense(Sense::click()));

        let rating = format!("{}/{}", marks.stars, crate::metadata::xmp::MAX_RATING);

        surface::with_menu(
            ui,
            &shown,
            surface::Subject::of("Rating", &rating),
            "The rating on this photograph.",
            |ui| {
                if ui
                    .button(format!("Show only {} stars and better", marks.stars))
                    .clicked()
                {
                    asked.push(BarAction::ShowOnlyStars(marks.stars));
                    ui.close();
                }
                if surface::more_settings(ui, Page::Marks) {
                    asked.push(BarAction::Settings("tags.sc_rating"));
                    ui.close();
                }
            },
        );
    }

    asked
}

/// A rating as filled stars, without the empty ones.
fn stars(rating: u8) -> String {
    "★".repeat(rating as usize)
}

fn jump_field(ui: &mut egui::Ui, status: &mut Status<'_>) -> Option<usize> {
    let response = ui.add_sized(
        Vec2::new(65., ui.available_height()),
        egui::TextEdit::singleline(status.jump_to).hint_text("go to"),
    );

    if status.asking_to_go_to {
        response.request_focus();
    }

    // Reachable by clicking, and by the key that asks for it. egui hands focus
    // to the next widget on Tab, and this is the first widget in the window —
    // so `Tab`, which means "the other pane" while comparing, landed in here
    // instead, and a text field with focus mutes every shortcut in the viewer.
    // The reasoning was sound and it left a control that could not be operated
    // without a mouse, so now something can ask.
    if response.gained_focus() && !response.clicked() && !status.asking_to_go_to {
        response.surrender_focus();
        return None;
    }

    if !(response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
        return None;
    }

    let typed = status.jump_to.parse::<usize>().ok();
    status.jump_to.clear();

    // The field is one based; positions outside the collection are ignored.
    typed
        .filter(|position| (1..=status.total).contains(position))
        .map(|position| position - 1)
}

/// Smallest and largest the slider reaches, as a percentage of the
/// photograph's own pixels.
///
/// The slider used to run from a tenth to ten times *the fitted size*, which
/// on a twenty-four megapixel photograph in a normal window could not reach
/// one-for-one at all: fitted is about a twelfth of native, so ten times
/// fitted is still less than actual size. It runs in the same percentages the
/// readout beside it shows now, logarithmically, so a drag covers the whole
/// range and the useful end of it is not squeezed into the first millimetre.
const MIN_PERCENT: f32 = 1.0;
const MAX_PERCENT: f32 = 1600.0;

fn zoom_slider(ui: &mut egui::Ui, percentage_zoom: f32, least_zoom: f32) -> Vec<Command> {
    // Before the first frame there is no magnification to show, and a slider
    // sitting at its floor would look like one.
    if percentage_zoom <= 0.0 {
        return Vec::new();
    }

    let (least, most) = ends(least_zoom);
    let mut percent = percentage_zoom.clamp(least, most);
    let slider = ui.add_sized(
        Vec2::new(200., ui.available_height()),
        crate::ui::slider::Fine::new(&mut percent, least..=most)
            .logarithmic(true)
            .show_value(false)
            .about("Zoom")
            .hint("How large the photograph is drawn.")
            .text("ð"),
    );

    if slider.changed() {
        return vec![Command::ZoomToPercent(percent, Anchor::FROM_THE_BAR)];
    }

    Vec::new()
}

/// The two ends of the rail, given whatever holds the zoom out.
///
/// A photograph small enough to be enlarged more than sixteen times fits at
/// more than the usual ceiling, and a rail whose ends were the wrong way round
/// would be a rail that does nothing at all; there the ceiling moves up instead
/// of the floor moving down, because the floor is the one the view will
/// actually enforce.
fn ends(least_zoom: f32) -> (f32, f32) {
    if !least_zoom.is_finite() || least_zoom <= MIN_PERCENT {
        return (MIN_PERCENT, MAX_PERCENT);
    }

    (least_zoom, MAX_PERCENT.max(least_zoom * 2.0))
}

fn zoom_label(ui: &mut egui::Ui, percentage_zoom: f32) -> Vec<Command> {
    let mut commands = Vec::new();

    let response = ui.add_sized(
        Vec2::new(45., ui.available_height()),
        egui::Label::new(format!("{percentage_zoom:.1}%")).sense(Sense::click()),
    );

    // Through the same helper as everything else. This was the last menu in
    // the program opening on the release, with no chevron, no hover text and
    // nothing saying what it was about — and `Response::context_menu` loses the
    // menu to a six-point drag, which on a figure this small is most of the
    // gestures aimed at it.
    let reading = format!("{percentage_zoom:.1}%");

    crate::ui::surface::with_menu(
        ui,
        &response,
        crate::ui::surface::Subject::of("Zoom", &reading),
        "How large the photograph is drawn.",
        |ui| {
            for (label, command) in [
                ("Fit to screen", Command::Fit),
                ("Fill screen", Command::Fill),
                ("Fit horizontal", Command::FitHorizontal),
                ("Fit vertical", Command::FitVertical),
            ] {
                if ui.button(label).clicked() {
                    commands.push(command);
                    ui.close();
                }
            }

            ui.separator();

            for percentage in PERCENTAGES {
                if ui.button(format!("{percentage:.0}%")).clicked() {
                    commands.push(Command::ZoomToPercent(*percentage, Anchor::FROM_THE_BAR));
                    ui.close();
                }
            }
        },
    );

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rating_is_shown_as_filled_stars() {
        assert_eq!(stars(0), "");
        assert_eq!(stars(3), "★★★");
        assert_eq!(stars(5).chars().count(), 5);
    }

    /// With nothing holding the zoom out, the rail runs the range it always
    /// ran.
    #[test]
    fn the_rail_runs_its_usual_range_when_nothing_holds_the_zoom() {
        assert_eq!(ends(0.0), (MIN_PERCENT, MAX_PERCENT));
        // A floor under the rail's own floor is no floor at all.
        assert_eq!(ends(MIN_PERCENT / 2.0), (MIN_PERCENT, MAX_PERCENT));
        assert_eq!(ends(f32::NAN), (MIN_PERCENT, MAX_PERCENT));
    }

    /// With the zoom held to fitting, the rail starts exactly where the zoom
    /// stops — so its left end is the fit rather than a stretch that asks for
    /// something the view will refuse.
    #[test]
    fn the_rail_starts_where_the_zoom_stops() {
        assert_eq!(ends(37.5), (37.5, MAX_PERCENT));
    }

    /// A photograph small enough to be enlarged past the usual ceiling gets a
    /// rail with its ends the right way round rather than no rail at all.
    #[test]
    fn a_floor_above_the_ceiling_raises_the_ceiling() {
        let (least, most) = ends(MAX_PERCENT + 400.0);

        assert_eq!(least, MAX_PERCENT + 400.0);
        assert!(most > least, "{least} to {most} is not a rail");
    }
}

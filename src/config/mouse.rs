//! What the pointer does, and where that is written down.
//!
//! The wheel is the loudest complaint in the whole viewer corpus, and the
//! instructive thing is what the complainants ask for. nomacs #237 ran from
//! 2018 to 2025 across sixteen accounts, at least one of whom said they had
//! uninstalled over it, and the line to design against is not "make it zoom"
//! but "just being able to bind it would be more than enough". Nobody is
//! asking to win the argument about the default; they are asking to be allowed
//! to lose it locally.
//!
//! So the gestures are few and fixed — which is the answer to IrfanView's
//! author, who has argued that "more options are not always a good move as they
//! make programs harder to support" — and it is the mapping that opens up. The
//! one field here that is not a mapping is `slider_travel`, and it is here for
//! the same reason: it is about the hand rather than about any one control.
//!
//! The wheel's two fields hold a *job* rather than a command, because a wheel
//! is an axis and one notch either way cannot be one command. Everything with
//! a single meaning — the double click, the middle button, the two thumb
//! buttons — holds the name of a command instead, from the one vocabulary in
//! [`VERBS`], so adding a gesture adds a row and not a vocabulary.
//!
//! The wheel's job in the contact sheet is deliberately not a field: the sheet
//! is an ordinary `ScrollArea` (`src/view/grid_view/mod.rs:289`) and scrolling
//! it is what a wheel does there. That is the argument for the image view
//! agreeing with the sheet about direction rather than the other way about.

use serde::{Deserialize, Serialize};

use super::registry::Choice;

/// What the pointer does.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct MouseConfig {
    /// What one notch over the photograph does.
    #[serde(default)]
    pub wheel: WheelJob,
    /// Whether the wheel runs the other way.
    ///
    /// One boolean, for the people whose muscle memory says wheel-up is
    /// forward. The complaint everywhere is that the direction cannot be
    /// reversed, not that it is wrong.
    #[serde(default)]
    pub wheel_reversed: bool,
    /// What one notch with Ctrl held does.
    #[serde(default = "zooms")]
    pub ctrl_wheel: WheelJob,
    /// Which button moves the photograph about.
    #[serde(default)]
    pub drag: DragButton,
    /// When the left button marks out a part of the photograph instead.
    #[serde(default)]
    pub mark_area: MarkArea,
    /// What two quick clicks on the photograph do.
    #[serde(default = "fit_or_actual")]
    pub double_click: String,
    /// What the wheel pressed as a button does. Nothing, by default.
    #[serde(default = "nothing")]
    pub middle: String,
    /// The thumb buttons, which mean back and forward everywhere else.
    #[serde(default = "previous")]
    pub back: String,
    #[serde(default = "next")]
    pub forward: String,
    /// How far the pointer travels to cross a slider, as a multiple of the
    /// rail's own length.
    ///
    /// The only field here that is a number rather than a mapping. It is under
    /// the mouse rather than under the window because it is a property of the
    /// hand rather than of any one control: the same person wants the same
    /// answer on every rail in the program, and gets it, because there is one
    /// value and every rail reads it.
    #[serde(default = "slider_travel")]
    pub slider_travel: f32,
}

fn zooms() -> WheelJob {
    WheelJob::Zoom
}

fn fit_or_actual() -> String {
    "fit_or_actual".to_string()
}

fn nothing() -> String {
    "nothing".to_string()
}

fn previous() -> String {
    "previous".to_string()
}

fn next() -> String {
    "next".to_string()
}

fn slider_travel() -> f32 {
    crate::ui::slider::drag::SHIPS_AS
}

impl Default for MouseConfig {
    fn default() -> Self {
        MouseConfig {
            wheel: WheelJob::default(),
            wheel_reversed: false,
            ctrl_wheel: zooms(),
            drag: DragButton::default(),
            mark_area: MarkArea::default(),
            double_click: fit_or_actual(),
            middle: nothing(),
            back: previous(),
            forward: next(),
            slider_travel: slider_travel(),
        }
    }
}

/// The one job a wheel notch does.
///
/// One at a time, which is the whole of the first fault: a notch used to call
/// *Next* and shove the photograph that had just arrived, in that order, with
/// nothing guarding the second against the first.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WheelJob {
    /// One photograph. Wheel-down is forward, as it is in every list.
    #[default]
    NextOrPrevious,
    /// Magnify about the pointer.
    Zoom,
    /// Move the photograph under the window.
    Pan,
    /// Nothing at all, deliberately.
    Nothing,
}

impl WheelJob {
    pub fn value(self) -> &'static str {
        match self {
            WheelJob::NextOrPrevious => "next_or_previous",
            WheelJob::Zoom => "zoom",
            WheelJob::Pan => "pan",
            WheelJob::Nothing => "nothing",
        }
    }

    /// The job of that name.
    ///
    /// A name this build does not know falls back to the caller's default
    /// rather than to nothing: nomacs #1281 is a wheel left doing nothing at
    /// all because the job it had been turned off from had no replacement, and
    /// a dead wheel is the one outcome nobody asked for.
    pub fn of(name: &str) -> Option<WheelJob> {
        [
            WheelJob::NextOrPrevious,
            WheelJob::Zoom,
            WheelJob::Pan,
            WheelJob::Nothing,
        ]
        .into_iter()
        .find(|job| job.value() == name)
    }
}

/// Which button drags the photograph about.
///
/// *Left* rather than the *any* the program used to do, and it is the one
/// place in this section where shipping today's behaviour would be wrong:
/// with every button panning, whether the right button opens a menu or moves
/// the photograph is decided by egui's `max_click_dist` of six points and
/// `max_click_duration` of eight tenths of a second. Move six points on the
/// way down and the menu never appears. *Any* stays a legal value for whoever
/// wants the old behaviour back.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DragButton {
    #[default]
    Left,
    Middle,
    Right,
    Any,
}

impl DragButton {
    pub fn value(self) -> &'static str {
        match self {
            DragButton::Left => "left",
            DragButton::Middle => "middle",
            DragButton::Right => "right",
            DragButton::Any => "any",
        }
    }

    pub fn of(name: &str) -> Option<DragButton> {
        [
            DragButton::Left,
            DragButton::Middle,
            DragButton::Right,
            DragButton::Any,
        ]
        .into_iter()
        .find(|button| button.value() == name)
    }
}

/// When the left button marks out a part of the photograph.
///
/// The default is the one that takes nothing away. With the whole photograph
/// on screen there is no slack to pan into and a left drag already moved
/// nothing at all, so that is the drag the marking is given; the moment there
/// is somewhere to pan to, the drag goes back to panning. Somebody who would
/// rather mark a magnified photograph and pan with the wheel pressed says
/// *Always*, and somebody who wants the old dead gesture back says *Never*.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarkArea {
    /// While the whole photograph is on screen, where a drag moved nothing.
    #[default]
    WhenItFits,
    /// Whatever the magnification. The photograph is then moved with the wheel
    /// pressed, or with the pan keys.
    Always,
    Never,
}

impl MarkArea {
    pub fn value(self) -> &'static str {
        match self {
            MarkArea::WhenItFits => "when_it_fits",
            MarkArea::Always => "always",
            MarkArea::Never => "never",
        }
    }

    pub fn of(name: &str) -> Option<MarkArea> {
        [MarkArea::WhenItFits, MarkArea::Always, MarkArea::Never]
            .into_iter()
            .find(|when| when.value() == name)
    }
}

/// The three answers, as the control draws them.
pub const MARK_AREAS: &[Choice] = &[
    Choice {
        value: "when_it_fits",
        label: "When the photograph fits the panel",
        sentence: "Where a drag would move nothing anyway. Magnified, the left button \
                   goes back to moving the photograph.",
    },
    Choice {
        value: "always",
        label: "Always",
        sentence: "The photograph is then moved with the wheel pressed, or with the pan keys.",
    },
    Choice {
        value: "never",
        label: "Never",
        sentence: "The left button then only ever moves the photograph, and nothing can be \
                   marked out at all.",
    },
];

/// The four jobs a wheel can be given, as the control draws them.
pub const WHEEL_JOBS: &[Choice] = &[
    Choice {
        value: "next_or_previous",
        label: "One photograph",
        sentence: "Wheel down goes forward, which is what it does in the contact sheet \
                   and in every list.",
    },
    Choice {
        value: "zoom",
        label: "Zoom",
        sentence: "About the pointer, so the point under it stays put.",
    },
    Choice {
        value: "pan",
        label: "Move the photograph",
        sentence: "Only when there is somewhere to move it to.",
    },
    Choice {
        value: "nothing",
        label: "Nothing",
        sentence: "",
    },
];

/// Which button the pan answers to.
pub const DRAG_BUTTONS: &[Choice] = &[
    Choice {
        value: "left",
        label: "The left button",
        sentence: "",
    },
    Choice {
        value: "middle",
        label: "The wheel",
        sentence: "",
    },
    Choice {
        value: "right",
        label: "The right button",
        sentence: "This is also the button the menus are on.",
    },
    Choice {
        value: "any",
        label: "Any button",
        sentence: "What the viewer used to do. A right drag then swallows the menu.",
    },
];

/// Everything a click, a press or a thumb button can be told to do.
///
/// One vocabulary for four fields. The names are what the file holds, so they
/// are the names a forum answer quotes; the labels are what the list says.
pub const VERBS: &[Choice] = &[
    Choice {
        value: "nothing",
        label: "Nothing",
        sentence: "",
    },
    Choice {
        value: "fit_or_actual",
        label: "Fit ↔ actual pixels",
        sentence: "Fitted to the panel, or at one screen pixel per photograph pixel.",
    },
    Choice {
        value: "fit",
        label: "Fit the photograph",
        sentence: "",
    },
    Choice {
        value: "fill",
        label: "Fill the panel",
        sentence: "",
    },
    Choice {
        value: "actual_pixels",
        label: "Actual pixels",
        sentence: "",
    },
    Choice {
        value: "zoom_in",
        label: "Zoom in",
        sentence: "",
    },
    Choice {
        value: "zoom_out",
        label: "Zoom out",
        sentence: "",
    },
    Choice {
        value: "next",
        label: "Next photograph",
        sentence: "",
    },
    Choice {
        value: "previous",
        label: "Previous photograph",
        sentence: "",
    },
    Choice {
        value: "next_stack",
        label: "Next run of frames",
        sentence: "",
    },
    Choice {
        value: "previous_stack",
        label: "Previous run of frames",
        sentence: "",
    },
    Choice {
        value: "page_forward",
        label: "Ten forward",
        sentence: "",
    },
    Choice {
        value: "page_back",
        label: "Ten back",
        sentence: "",
    },
    Choice {
        value: "first",
        label: "The first photograph",
        sentence: "",
    },
    Choice {
        value: "last",
        label: "The last photograph",
        sentence: "",
    },
    Choice {
        value: "fullscreen",
        label: "Fill the screen",
        sentence: "",
    },
    Choice {
        value: "contact_sheet",
        label: "The contact sheet",
        sentence: "",
    },
    Choice {
        value: "filmstrip",
        label: "The strip of thumbnails",
        sentence: "",
    },
    Choice {
        value: "keywords",
        label: "The keyword panel",
        sentence: "",
    },
    Choice {
        value: "information",
        label: "The information panel",
        sentence: "",
    },
    Choice {
        value: "filter",
        label: "The filter bar",
        sentence: "",
    },
    Choice {
        value: "compare",
        label: "Compare with the neighbours",
        sentence: "",
    },
    Choice {
        value: "overlay",
        label: "Move what is written over the photograph",
        sentence: "",
    },
    Choice {
        value: "marks",
        label: "Show what has clipped",
        sentence: "",
    },
    Choice {
        value: "turn_left",
        label: "Turn anticlockwise",
        sentence: "",
    },
    Choice {
        value: "turn_right",
        label: "Turn clockwise",
        sentence: "",
    },
    Choice {
        value: "keep",
        label: "Mark as a keeper",
        sentence: "",
    },
    Choice {
        value: "reject",
        label: "Mark as a reject",
        sentence: "",
    },
    Choice {
        value: "move_to",
        label: "Send it somewhere",
        sentence: "",
    },
    Choice {
        value: "copy_to",
        label: "Copy it somewhere",
        sentence: "",
    },
    Choice {
        value: "to_rejected_folder",
        label: "Send it to the rejects folder",
        sentence: "",
    },
    Choice {
        value: "delete",
        label: "Send it to the bin",
        sentence: "",
    },
    Choice {
        value: "undo",
        label: "Put the last thing back",
        sentence: "",
    },
    Choice {
        value: "keys",
        label: "Show the keys",
        sentence: "",
    },
    Choice {
        value: "settings",
        label: "Open the settings",
        sentence: "",
    },
    Choice {
        value: "exit",
        label: "Close the viewer",
        sentence: "nomacs #864 asked for exactly this on the middle button.",
    },
];

/// Whether `name` is a verb this build knows.
pub fn names_a_verb(name: &str) -> bool {
    VERBS.iter().any(|verb| verb.value == name)
}

/// The verb of that name, or `nothing` — never an unbound gesture that looks
/// bound in the list.
pub fn verb_or_nothing(name: &str) -> &'static str {
    VERBS
        .iter()
        .find(|verb| verb.value == name)
        .map(|verb| verb.value)
        .unwrap_or("nothing")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_defaults_are_the_ones_the_plan_argues_for() {
        let mouse = MouseConfig::default();

        assert_eq!(mouse.wheel, WheelJob::NextOrPrevious);
        assert_eq!(mouse.ctrl_wheel, WheelJob::Zoom);
        assert_eq!(
            mouse.drag,
            DragButton::Left,
            "not `any`: see the note above"
        );
        assert_eq!(mouse.double_click, "fit_or_actual");
        assert_eq!(mouse.middle, "nothing");
        assert_eq!(mouse.back, "previous");
        assert_eq!(mouse.forward, "next");
    }

    /// Every default names something the program can actually do, which is the
    /// failure a string-valued field invites.
    #[test]
    fn every_default_names_a_verb() {
        let mouse = MouseConfig::default();

        for name in [
            &mouse.double_click,
            &mouse.middle,
            &mouse.back,
            &mouse.forward,
        ] {
            assert!(names_a_verb(name), "{name} is not a verb");
        }
    }

    #[test]
    fn a_verb_that_does_not_exist_becomes_nothing() {
        assert_eq!(verb_or_nothing("open_the_pod_bay_doors"), "nothing");
        assert_eq!(verb_or_nothing("fullscreen"), "fullscreen");
    }

    #[test]
    fn no_two_verbs_share_a_name() {
        for (at, verb) in VERBS.iter().enumerate() {
            assert!(
                !VERBS[..at].iter().any(|other| other.value == verb.value),
                "{} is in the list twice",
                verb.value
            );
        }
    }

    #[test]
    fn the_wheel_jobs_and_the_drag_buttons_round_trip() {
        for choice in WHEEL_JOBS {
            let job = WheelJob::of(choice.value).expect("the control offers it");
            assert_eq!(job.value(), choice.value);
        }

        for choice in DRAG_BUTTONS {
            let button = DragButton::of(choice.value).expect("the control offers it");
            assert_eq!(button.value(), choice.value);
        }
    }

    #[test]
    fn the_section_round_trips_through_json() {
        let mouse = MouseConfig {
            wheel: WheelJob::Zoom,
            wheel_reversed: true,
            middle: "fullscreen".to_string(),
            ..MouseConfig::default()
        };

        let json = serde_json::to_string(&mouse).unwrap();
        assert_eq!(serde_json::from_str::<MouseConfig>(&json).unwrap(), mouse);
        assert!(json.contains("\"zoom\""), "{json}");
    }

    /// A section written by an older build has none of these keys, and the
    /// defaults are the right answer for every one of them.
    #[test]
    fn an_empty_section_is_the_defaults() {
        let mouse: MouseConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(mouse, MouseConfig::default());
    }
}

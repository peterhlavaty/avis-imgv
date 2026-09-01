//! `mouse.*`: what the pointer does.
//!
//! Eight rows, all on one page, which is the point. nomacs's own thread about
//! the wheel ended with a collaborator noting that two checkboxes buried in
//! two different places would already have swapped the functions — after seven
//! years in which nobody in the thread had found them — and conceding that the
//! mouse "needs a refactor and should have their own settings section".

use super::*;

use crate::config::mouse::{
    DragButton, MarkArea, WheelJob, DRAG_BUTTONS, MARK_AREAS, VERBS, WHEEL_JOBS,
};

/// A row holding the name of a command.
///
/// The getter maps whatever the file holds onto one of the names the list
/// offers, so a hand-edited typo shows as *Nothing* rather than as a blank
/// control claiming to be set to something.
macro_rules! verb {
    ($($field:tt)+) => {
        Access::Enum {
            get: |c| crate::config::mouse::verb_or_nothing(&c.$($field)+),
            set: |c, v| {
                if crate::config::mouse::names_a_verb(v) {
                    c.$($field)+ = v.to_string();
                }
            },
            choices: VERBS,
        }
    };
}

pub fn rows() -> Vec<Row> {
    vec![
        row!(
            KeysAndMouse / Mouse,
            "mouse.wheel",
            "The wheel over the photograph",
            "One job at a time. A notch used to move to the next photograph and shove \
             the one that had just arrived, in that order.",
            [
                "wheel",
                "scroll",
                "mouse wheel",
                "zoom with wheel",
                "scroll_navigation"
            ],
            Live,
            None,
            Access::Enum {
                get: |c| c.mouse.wheel.value(),
                set: |c, v| {
                    if let Some(job) = WheelJob::of(v) {
                        c.mouse.wheel = job;
                    }
                },
                choices: WHEEL_JOBS,
            },
        ),
        row!(
            KeysAndMouse / Mouse,
            "mouse.wheel_reversed",
            "Turn the wheel round",
            "Wheel up goes forward instead of back. The complaint everywhere is that \
             the direction cannot be reversed, not that it is wrong.",
            ["reverse", "invert", "backwards", "direction"],
            Live,
            None,
            boolean!(mouse.wheel_reversed),
        ),
        row!(
            KeysAndMouse / Mouse,
            "mouse.ctrl_wheel",
            "Ctrl and the wheel",
            "Zoom, by convention: it is what a scrolling view does everywhere, and \
             what this viewer already did.",
            ["ctrl wheel", "control wheel", "zoom"],
            Live,
            None,
            Access::Enum {
                get: |c| c.mouse.ctrl_wheel.value(),
                set: |c, v| {
                    if let Some(job) = WheelJob::of(v) {
                        c.mouse.ctrl_wheel = job;
                    }
                },
                choices: WHEEL_JOBS,
            },
            explained: "Shift and the wheel move ten photographs at a time, and Alt and the \
                        wheel move the photograph sideways. Neither is a setting: they are \
                        the same step the page keys use, on the same axis.",
        ),
        row!(
            KeysAndMouse / Mouse,
            "mouse.drag",
            "Move the photograph with",
            "Which button drags the photograph about. Every button used to, so a right \
             drag moved the picture and swallowed the menu it was aimed at.",
            ["drag", "pan", "button", "right click"],
            Live,
            None,
            Access::Enum {
                get: |c| c.mouse.drag.value(),
                set: |c, v| {
                    if let Some(button) = DragButton::of(v) {
                        c.mouse.drag = button;
                    }
                },
                choices: DRAG_BUTTONS,
            },
            explained: "The wheel pressed and dragged always moves the photograph, whatever \
                        this says and whether or not there is slack, so a fitted photograph \
                        is not a dead surface.",
        ),
        row!(
            KeysAndMouse / Mouse,
            "mouse.mark_area",
            "Mark out part of the photograph with",
            "Dragging the left button draws a rectangle on the photograph, whose sides \
             can then be taken hold of, which a click magnifies to and a second click \
             clears.",
            ["crop", "marquee", "selection", "rectangle", "marked area", "zoom to selection"],
            Live,
            None,
            Access::Enum {
                get: |c| c.mouse.mark_area.value(),
                set: |c, v| {
                    if let Some(when) = MarkArea::of(v) {
                        c.mouse.mark_area = when;
                    }
                },
                choices: MARK_AREAS,
            },
            explained: "It never takes the drag away from moving the photograph: with the \
                        default, the left button marks only while the whole photograph is on \
                        screen and there is no slack to pan into.",
        ),
        row!(
            KeysAndMouse / Mouse,
            "mouse.double_click",
            "Two clicks on the photograph",
            "Three separate requests have been filed against this click in other \
             viewers, disagreeing about what it should do — which is the argument for \
             making it a setting rather than for picking one.",
            ["double click", "double-click", "two clicks", "fullscreen"],
            Live,
            None,
            verb!(mouse.double_click),
        ),
        row!(
            KeysAndMouse / Mouse,
            "mouse.middle",
            "The wheel pressed",
            "Nothing, by default. GNOME advises against relying on the middle button \
             at all and every shipping viewer disagrees with it; a binding that is \
             empty until somebody fills it satisfies both.",
            ["middle click", "middle button", "wheel click"],
            Live,
            None,
            verb!(mouse.middle),
        ),
        row!(
            KeysAndMouse / Mouse,
            "mouse.back",
            "The back thumb button",
            "It fires when the button goes down, never on the release: a viewer that \
             waits to see whether a side-button click was a double makes walking a \
             folder feel slow and still moves one frame.",
            ["back", "thumb", "side button", "extra"],
            Live,
            None,
            verb!(mouse.back),
        ),
        row!(
            KeysAndMouse / Mouse,
            "mouse.forward",
            "The forward thumb button",
            "Whether these arrive at all depends on the mouse and its driver, so the \
             row is here even on a machine where nothing ever comes in on it.",
            ["forward", "thumb", "side button", "extra"],
            Live,
            None,
            verb!(mouse.forward),
        ),
    ]
}

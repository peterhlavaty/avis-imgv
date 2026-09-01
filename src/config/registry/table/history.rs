//! The rows for `history.*`: what is kept, what undo stops on, and the keys.

use super::*;

const CLASSES: &[Choice] = &[
    Choice {
        value: "view",
        label: "Where you were",
        sentence: "The mode, the panels, the photograph you were on, the zoom, the narrowing.",
    },
    Choice {
        value: "settings",
        label: "Settings",
        sentence: "Anything changed in this window.",
    },
    Choice {
        value: "content",
        label: "Photographs",
        sentence: "Stars, flags, labels, keywords, turns, and moving or deleting a file.",
    },
];

pub fn rows() -> Vec<Row> {
    vec![
        row!(
            History / Plain,
            "history.remember",
            "Actions to remember",
            "How many of the things you have done are kept, or nought for all of them, \
             which is what it is set to. A deed is a handful of paths and a small \
             document rather than a photograph, so a whole day of culling is a few \
             kilobytes; a limit is here for somebody who would rather the list stayed \
             short enough to read to the end.",
            ["undo", "redo", "history", "depth", "steps", "how many", "limit"],
            Live,
            None,
            whole!(usize, 0, 100000, "", true, history.remember),
        ),
        row!(
            History / Plain,
            "history.undoes",
            "One press of undo stops on",
            "Everything is remembered whatever is ticked here, and everything is in the \
             history panel and can be gone back to by clicking it. What a tick decides \
             is only whether one press of undo comes to *rest* there. With the first \
             unticked, undo after twenty photographs walked past still lands on the \
             rating rather than twenty presses short of it — and where you were goes \
             back with it, because all of it did happen.",
            ["undo", "redo", "skip", "classes", "what undo does", "stop"],
            Live,
            None,
            Access::Flags {
                get: |c, name| c.history.undoes.get(name),
                set: |c, name, on| c.history.undoes.set(name, on),
                options: CLASSES,
            },
        ),
        row!(
            History / Plain,
            "history.merge_within_ms",
            "Count nudges this close together as one",
            "A wheel turned twice, or an arrow held down, arrives once a frame. Two \
             that land within this of each other become one line in the history, so \
             that one press of undo is worth a whole gesture rather than a sixtieth of \
             a second. Nought switches it off and lists every notch. A drag is one \
             line whatever this says, because nothing is recorded until the button \
             comes up.",
            ["undo", "history", "merge", "coalesce", "gesture", "repeat"],
            Live,
            None,
            whole!(u64, 0, 5000, " ms", true, history.merge_within_ms),
        ),
        row!(
            History / Plain,
            "history.panel_visible",
            "Show the history panel",
            "The list of what you have done, down the right-hand side. The same thing \
             the key does, and the same thing the second button on the panel itself \
             offers; however it is changed it is written here, so the next launch opens \
             with it as it was left.",
            ["history", "panel", "show", "hide", "list", "undo"],
            Live,
            None,
            boolean!(history.panel_visible),
        ),
        row!(
            History / Plain,
            "history.panel_width",
            "How wide the history panel is",
            "The list of what you have done, down the right-hand side. Dragging its \
             edge writes this, so it is here to be read rather than because dragging \
             is the hard way to do it.",
            ["history", "panel", "width", "list"],
            Live,
            None,
            decimal!(180.0, 640.0, " pt", true, history.panel_width),
        ),
        row!(
            KeysAndMouse / Keys,
            "history.sc_undo",
            "Undo",
            "Take back the last thing done, whatever kind of thing it was.",
            ["undo", "back", "mistake", "revert"],
            Live,
            Everywhere,
            key!(history.sc_undo),
        ),
        row!(
            KeysAndMouse / Keys,
            "history.sc_redo",
            "Redo",
            "Do again the thing that was last taken back. Going back never throws \
             anything away, so this is still here after going somewhere else and \
             coming back.",
            ["redo", "again", "forward"],
            Live,
            Everywhere,
            key!(history.sc_redo),
        ),
        row!(
            KeysAndMouse / Keys,
            "history.sc_panel",
            "The history panel",
            "Show or hide the list of everything done this run. A row can be clicked \
             to go back to it, and the second button on a row offers to carry that one \
             out again where you are now.",
            ["history", "panel", "undo", "list", "what I did"],
            Live,
            Everywhere,
            key!(history.sc_panel),
        ),
    ]
}

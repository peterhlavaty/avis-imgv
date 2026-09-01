//! The rows for `history.*`: what is kept, and the two keys that walk it.

use super::*;

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
    ]
}

//! `cull.*`: where photographs are sent, and the keys that send them.

use super::*;

pub fn rows() -> Vec<Row> {
    let mut rows = vec![
        row!(
            MovingAndDeleting / Destinations,
            "cull.destinations",
            "Where photographs go",
            "Folders reached by a digit while the move or copy panel is up. There are \
             nine digits and the digit is the gesture, so the first nine keep theirs \
             and the rest are reached with the arrow keys.",
            ["move to", "copy to", "folders", "selects", "sort into"],
            Live,
            None,
            Access::Records(List::Destinations, |c| c.cull.destinations.len()),
        ),
        row!(
            MovingAndDeleting / Destinations,
            "cull.rejected_folder",
            "The folder rejects go to",
            "A folder name, made beside the photographs, for the frames that are not \
             staying. It is what a memory card or a network share has instead of a bin. \
             Left empty, the key that sends things there does nothing at all.",
            [
                "reject",
                "rejects",
                "where do rejects go",
                "trash",
                "bin",
                "cull"
            ],
            Live,
            None,
            text!(cull.rejected_folder),
        ),
    ];

    rows.extend(keys());
    rows
}

fn keys() -> Vec<Row> {
    vec![
        row!(KeysAndMouse / Keys, "cull.sc_move", "Move to…",
            "Send the picture on screen to one of the folders on the panel.",
            ["move", "sort", "file"], Live, Everywhere, key!(cull.sc_move)),
        row!(KeysAndMouse / Keys, "cull.sc_copy", "Copy to…",
            "Put a copy of it in one of them, leaving the photograph where it is.",
            ["copy", "duplicate"], Live, Everywhere, key!(cull.sc_copy)),
        row!(KeysAndMouse / Keys, "cull.sc_reject_folder", "To the rejected folder",
            "Move it into the folder for the frames that are not staying, which is what a card or a network share has instead of a bin.",
            ["reject", "rejects", "cull"], Live, Everywhere, key!(cull.sc_reject_folder)),
    ]
}

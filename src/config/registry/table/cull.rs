//! `cull.*`: where photographs are sent, and the keys that send them.

use super::*;

/// Which bin the delete key means.
const BINS: &[Choice] = &[
    Choice {
        value: "system",
        label: "The platform's bin",
        sentence: "What Delete means in every other program, which is why it is the \
                   default. It does not reach a memory card or a share over the \
                   network, and on macOS nothing can take anything back out of it.",
    },
    Choice {
        value: "folder",
        label: "A folder of the viewer's own",
        sentence: "A folder like any other, which is the point: it opens in this \
                   viewer, so what an hour of culling threw away can be looked \
                   through before any of it is really gone. It remembers where each \
                   photograph came from, it reaches a card, and emptying it deletes \
                   the folder.",
    },
];

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
        row!(
            MovingAndDeleting / Destinations,
            "cull.bin",
            "What the delete key means",
            "Where a photograph goes when it is thrown out. The platform's bin is the \
             default because that is what nearly everybody expects; the viewer's own \
             folder is the answer for a card, for a network share, and for wanting to \
             look through what an hour of culling threw away.",
            ["bin", "trash", "recycle", "delete", "wastebasket"],
            Live,
            None,
            Access::Enum {
                get: |c| {
                    BINS.iter()
                        .find(|choice| choice.value == c.cull.bin)
                        .map(|choice| choice.value)
                        .unwrap_or("system")
                },
                set: |c, v| c.cull.bin = v.to_string(),
                choices: BINS,
            },
        ),
        row!(
            MovingAndDeleting / Destinations,
            "cull.bin_folder",
            "Where that folder is",
            "An absolute path. Left empty it is a folder called bin beside the \
             viewer's own files. One bin rather than one per shoot: a path relative \
             to the open folder would be a different bin in every folder, and the \
             question asked on the way out would be about whichever one happened to \
             be open.",
            ["bin", "trash", "folder", "where", "recycle"],
            Live,
            None,
            optional_text!(cull.bin_folder),
        ),
        row!(
            MovingAndDeleting / Confirmations,
            "cull.ask_to_empty_the_bin",
            "Closing with a full bin",
            "With something still in the viewer's own bin, closing it asks whether to \
             empty it first. Turned off, the bin is simply kept — which is what \
             somebody who treats it as a holding folder wants. Emptying is confirmed \
             either way; that one is not a setting.",
            ["bin", "empty", "exit", "quit", "closing", "ask"],
            Live,
            None,
            boolean!(cull.ask_to_empty_the_bin),
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
        row!(KeysAndMouse / Keys, "cull.sc_put_back", "Put it back",
            "Take it out of the viewer's own bin and put it back in the folder it was thrown out of. Nothing, for a photograph that is not in the bin.",
            ["restore", "put back", "undelete", "bin", "recover"], Live, Everywhere, key!(cull.sc_put_back)),
        row!(KeysAndMouse / Keys, "cull.sc_reject_folder", "To the rejected folder",
            "Move it into the folder for the frames that are not staying, which is what a card or a network share has instead of a bin.",
            ["reject", "rejects", "cull"], Live, Everywhere, key!(cull.sc_reject_folder)),
    ]
}

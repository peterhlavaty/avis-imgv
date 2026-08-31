//! The keys the program reads and does not let anybody change.
//!
//! They are entered here rather than left out for two reasons: the clash
//! checker cannot see a collision with a key it has never heard of, and a
//! search for "cheat sheet" has to find one. A row whose access is `Fixed` is
//! drawn read-only, greyed, with the key it is bound to.

use super::*;

/// One read-only row.
macro_rules! fixed {
    ($path:literal, $key:literal, $label:literal, $sentence:literal, [$($alias:literal),* $(,)?], $scope:ident) => {
        Row {
            page: Page::KeysAndMouse,
            group: Group::Keys,
            path: $path,
            label: $label,
            sentence: $sentence,
            aliases: &[$($alias),*],
            access: Access::Fixed($key),
            effect: Effect::None,
            scope: Scope::$scope,
        }
    };
}

pub fn rows() -> Vec<Row> {
    vec![
        fixed!(
            "fixed.cheat_sheet",
            "?",
            "Show the keys",
            "The list of what every key does in the mode on screen. Not configurable: \
             it is the key every program uses for this, and somebody who cannot \
             remember the keys cannot look up the key for looking up the keys.",
            ["help", "cheat sheet", "keys", "shortcuts", "?"],
            Everywhere
        ),
        fixed!(
            "fixed.frame_timings",
            "F10",
            "Frame timings",
            "Show or hide the strip saying how long each frame took.",
            ["fps", "performance", "timings", "debug"],
            Everywhere
        ),
        fixed!(
            "fixed.escape",
            "Escape",
            "Get the keyboard back",
            "Takes the keyboard away from whatever text field has it, closes an \
             overlay, and answers no to a question. It means six things, and always \
             the safe one.",
            ["escape", "cancel", "back", "stuck"],
            Everywhere
        ),
        fixed!(
            "fixed.first",
            "Home",
            "First photograph",
            "The first photograph on show, after the filter and the stacking.",
            ["home", "first", "start"],
            ImageView
        ),
        fixed!(
            "fixed.last",
            "End",
            "Last photograph",
            "The last photograph on show.",
            ["end", "last"],
            ImageView
        ),
        fixed!(
            "fixed.page_back",
            "PageUp",
            "A screenful back",
            "Walks a long folder quickly: as many photographs as are shown side by \
             side, at once.",
            ["page up", "skip", "jump"],
            ImageView
        ),
        fixed!(
            "fixed.page_forward",
            "PageDown",
            "A screenful forward",
            "The same, forwards.",
            ["page down", "skip", "jump"],
            ImageView
        ),
        fixed!(
            "fixed.next_pane",
            "Tab",
            "The other pane",
            "While photographs are pinned side by side, moves which one the keys are \
             about. Its pane carries a border so it is unmistakable.",
            ["tab", "compare", "pane", "focus"],
            ImageView
        ),
        fixed!(
            "fixed.drop_pane",
            "/",
            "Drop this pane",
            "Takes the focused photograph out of a comparison, leaving the others to \
             re-tile. Read without modifiers, which on the Slovak, German and French \
             layouts makes it hard to press.",
            ["compare", "close", "remove", "slash"],
            ImageView
        ),
        fixed!(
            "fixed.grid_arrows",
            "Arrows",
            "Move about the contact sheet",
            "One cell in each direction. With Ctrl held they step over a run of frames \
             rather than through it.",
            ["arrows", "cursor", "move", "navigate"],
            Gallery
        ),
        fixed!(
            "fixed.grid_open",
            "Enter",
            "Open the cell under the cursor",
            "Shows that photograph on its own.",
            ["enter", "open", "return"],
            Gallery
        ),
        fixed!(
            "fixed.grid_back",
            "Backspace",
            "Back to the contact sheet",
            "Returns from a photograph opened out of the sheet.",
            ["backspace", "back", "return"],
            Gallery
        ),
        fixed!(
            "fixed.tree_keys",
            "Arrows · Space · Enter",
            "Move about the folder tree",
            "Up and down move the highlight, right and left open and close a folder, \
             Space folds it, and Enter opens it.",
            ["tree", "folders", "navigate"],
            Overlay
        ),
        fixed!(
            "fixed.destination_digits",
            "1 – 9 · Enter",
            "Pick a destination",
            "While the move or copy panel is up, a digit sends the photograph to that \
             slot and Enter repeats the last one. There are nine digits and the digit \
             is the gesture, which is why the tenth destination has no key.",
            ["digits", "numbers", "move to", "copy to", "slot"],
            Overlay
        ),
        fixed!(
            "fixed.context_menu",
            "Shift + F10",
            "Open the menu for what has the keyboard",
            "The keyboard route to the second button's menu. egui cannot read the \
             dedicated Menu key at all — its key list runs F1 to F35 and has no entry \
             for it — so this is the only route there is.",
            ["context menu", "right click", "menu key"],
            Everywhere
        ),
    ]
}

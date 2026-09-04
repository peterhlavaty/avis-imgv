//! What the viewer can be asked to do, and what a key press turns into.
//!
//! The vocabulary, with no reader in it. `app::input` reads a frame's keys and
//! answers in these words; the menus, the status bar, the cheat sheet and the
//! history all name them too, and none of those should have to reach up into
//! the shell to do it.
//!
//! It sat in `app::input` beside the reader, which meant `ui`, `view` and
//! `history` each imported from `app` — three cycles, for an enum that carries
//! nothing but a `Mode` and a `Flag`.

use crate::metadata::xmp::Flag;
use crate::mode::Mode;

/// Something the application can be asked to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Exit,
    ToggleGrid,
    /// Move to the next mode round.
    NextMode,
    /// Go straight to one, as the menu does.
    SetMode(Mode),
    ToggleMenu,
    ToggleSidePanel,
    ToggleMetrics,
    ToggleFlatten,
    ToggleWatcher,
    ToggleTagPanel,
    /// Put this many stars on the image on screen.
    SetRating(u8),
    /// Keep it, throw it out, or take the mark back off.
    SetFlag(Flag),
    /// Put this colour label on it, by its position in [`Label::CHOICES`].
    SetLabel(usize),
    /// Move to the next photograph after every mark, or stop doing that.
    ToggleAdvance,
    /// Turn it a quarter, clockwise or the other way.
    ///
    /// Written to the sidecar and never to the photograph: a raw file cannot
    /// be rewritten without losing something, and a JPEG re-encoded is a JPEG
    /// made worse. It is the most-expected verb after delete and the one most
    /// often implemented by quietly modifying the file.
    Turn(bool),
    /// Send the picture on screen to the platform's bin.
    Delete,
    /// Delete it outright, which is asked about first.
    DeletePermanently,
    /// Take it out of the viewer's own bin and put it back where it came from.
    PutBack,
    /// Fill the screen, or give it back.
    ToggleFullscreen,
    /// Show or hide the bar that narrows and orders the folder.
    ToggleFilter,
    /// Set the rules aside without forgetting them, or put them back.
    SuspendFilter,
    /// Send the photograph somewhere else, or make a copy of it there.
    MoveTo,
    CopyTo,
    /// Move it into the folder for the frames that are not staying.
    ToRejectedFolder,
    /// Put back whatever the last thing did.
    /// Take back the last thing done.
    Undo,
    /// Do again the thing that was last taken back.
    Redo,
    /// Show or hide the list of what has been done.
    ToggleHistoryPanel,
    /// Show the keys, for the mode that is on screen.
    ShowKeys,
    /// Open the whole settings window.
    ShowSettings,
    /// Open the menu for whatever last had the keyboard.
    ///
    /// The keyboard route to the second button. egui cannot read the dedicated
    /// Menu key at all — its key list runs F1 to F35 and grepping it for `Menu`
    /// returns nothing — so this is the only route there is.
    ContextMenu,
    /// Show or hide the strip of thumbnails under the photograph.
    ToggleFilmstrip,
    /// Put every picked-out photograph back.
    PickNoneOut,
    /// Show the folder stacked, or put every frame back.
    ToggleStacking,
    /// Open or close the stack the cursor is in.
    ToggleStack,
    /// Change which frame stands for that stack.
    StandingBack,
    StandingForward,
    /// Step over a run of frames rather than through it.
    PreviousStack,
    NextStack,
}

impl Command {
    /// Whether this is a mark, and so whether it may advance to the next
    /// photograph once it has been applied.
    /// Whether this is one of the marking commands.
    ///
    /// Read only by the reader in `app::input`, which the `gui` feature gates,
    /// so it is dead without a window rather than dead altogether.
    #[cfg_attr(not(feature = "gui"), allow(dead_code))]
    pub(crate) fn is_a_mark(self) -> bool {
        matches!(
            self,
            Command::SetRating(_) | Command::SetFlag(_) | Command::SetLabel(_)
        )
    }
}

/// Overlays that take over the keyboard while open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    Navigator,
    DirectoryTree,
}

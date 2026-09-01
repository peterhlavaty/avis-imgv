//! Asking the program what it looks like, once at the foot of the frame.
//!
//! This is the only place anything is recorded, which is the point. There are
//! five dispatchers and a filter bar that goes round all of them; a rule
//! written at each would be six copies of one rule, and the seventh route
//! somebody adds next year would record nothing without anybody noticing.
//!
//! Everything here is a borrow. [`Watched`] holds references to the two things
//! that are not scalars, so building it allocates nothing, and the comparison
//! against the last frame allocates nothing either. A clone happens on the
//! frames where something moved, and on no others.

use std::time::Duration;

use eframe::egui;

use crate::history::{Panels, Watched};

use super::super::App;

impl App {
    /// Records whatever moved since the last frame.
    ///
    /// Called last in the frame, after every command has been carried out and
    /// every view has drawn, so that what it sees is where the frame *ended*
    /// rather than a state half way through being changed.
    pub(in crate::app) fn watch_history(&mut self, ctx: &egui::Context) {
        // A drag is one gesture and deserves one row. Nothing is looked at at
        // all while the pointer is down — not the view and not the settings —
        // so the comparison when the hand comes off is against where the
        // gesture began. This is the same rule the settings window already
        // follows for the fields the caches are built from, and it is what
        // keeps a panel dragged wider from being one row of history per frame:
        // `remember_runtime` writes that width into the configuration as the
        // hand moves.
        if ctx.input(|i| i.pointer.any_down()) {
            return;
        }

        let watched = Watched {
            folder: &self.base_path,
            mode: self.mode,
            panels: Panels {
                menu: self.menu_visible,
                side: self.side_panel_visible,
                metrics: self.metrics_visible,
                tags: self.tag_panel_visible,
                filter: self.filter_visible,
                filmstrip: self.filmstrip_visible,
            },
            // The store position rather than the position in what is shown, so
            // that coming back to it survives the filter having moved.
            cursor: self.image_view.selected_index(),
            place: self.image_view.place(),
            columns: self.grid_view.columns(),
            flattened: self.flattened,
            advancing: self.advancing,
            selection: self.grid_view.selection(),
            narrowing: &self.narrowing,
        };

        let seen = self.watching.look(&watched);
        let within = Duration::from_millis(self.settings.history.merge_within_ms);

        if let Some(changes) = seen {
            self.history.note(changes, within);
        }

        // The configuration is looked at separately, and only on the frames
        // where it has been written. It is not in the snapshot because only
        // the registry knows how to compare it, and walking those hundred and
        // eighty rows every frame measured at ten microseconds of every frame
        // — for an answer that is "nothing moved" all but a handful of times
        // in a session. Every deliberate change ends at `save_settings`, so
        // that is what says when to look.
        //
        // Never folded into the row in front of it: a slider dragged in the
        // settings window is already one gesture by the time it arrives here,
        // and two settings changed in quick succession are two decisions.
        if std::mem::take(&mut self.settings_touched) {
            if let Some(change) = self.watching.look_at_settings(&self.settings) {
                self.history.note(vec![change], Duration::ZERO);
            }
        }

        self.watching.done_looking();
    }
}

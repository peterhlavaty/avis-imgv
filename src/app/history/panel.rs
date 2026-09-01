//! Drawing the list of what was done, and doing what it is told.

use eframe::egui;

use crate::history::{panel, Action, Deed, Way};

use super::super::App;

impl App {
    /// Draws the panel, if it is up, and carries out what was clicked.
    pub(in crate::app) fn show_history_panel(&mut self, ctx: &egui::Context) {
        let width = self.settings.history.panel_width;

        // The toggle has to change a pixel whatever the state, or the key
        // looks broken on a fresh start.
        if self.history.is_empty() {
            panel::nothing_yet(ctx, self.history_panel_visible, width);
            return;
        }

        let actions = panel::ui(
            ctx,
            self.history_panel_visible,
            width,
            &mut self.history_panel,
            &self.history,
        );

        for action in actions {
            match action {
                Action::GoTo(id) => self.go_to_in_history(id),
                Action::Repeat(id) => self.repeat_in_history(id),
                Action::Settings(path) => self.open_settings_at(path),
                Action::Width(width) => {
                    self.settings.history.panel_width = width;
                    self.save_settings();
                }
            }
        }
    }

    /// Goes back — or forward — to a row chosen in the panel.
    ///
    /// A click names the row it means, so no class is stepped over here: the
    /// answer to "take me back to this one" is that one and everything between.
    pub(super) fn go_to_in_history(&mut self, id: crate::history::NodeId) {
        let route = self.history.plan_go_to(id);
        self.consider(route, Way::Back);
    }

    /// Carries one row out again, here, and files it as the latest thing done.
    ///
    /// Deliberately not a jump: the program stays where it is and that one deed
    /// happens on top of it, which is what "do only this one" means. It joins
    /// the history under the cursor like anything else, so it can be undone in
    /// its turn.
    fn repeat_in_history(&mut self, id: crate::history::NodeId) {
        let Some(deed) = self.history.entry(id).map(|entry| entry.deed.clone()) else {
            return;
        };

        match deed {
            Deed::Start => {}
            Deed::Files(step) => {
                // The recording says what the marks were when it first
                // happened, and they are not that now. Undoing this repeat has
                // to put back what is there at this moment, so the half that
                // says "before" is taken again.
                let step = self.as_it_stands(step);
                let said = step.describe(Way::Forward);
                let done = step.run(Way::Forward);

                self.history.record(Deed::Files(step));
                self.watching.resync();
                self.settle(done, said);
            }
            Deed::Changed(changes) => {
                for change in &changes {
                    self.restore(change, Way::Forward);
                }

                // No resync, deliberately. The watcher looks at the foot of
                // this frame, sees what actually moved, and files it with the
                // "before" taken from where the program actually was — which
                // is what makes undoing the repeat correct rather than a
                // second-hand copy of an older row's idea of before.
            }
        }
    }

    /// The same step, with its "before" taken as things stand now.
    fn as_it_stands(&mut self, step: crate::history::Step) -> crate::history::Step {
        use crate::history::Step;

        match step {
            Step::Marked { image, after, .. } => {
                self.load_annotations(&image);
                let before = self.annotations.get(&image, None).clone();

                Step::Marked {
                    image,
                    before: Box::new(before),
                    after,
                }
            }
            Step::Many(steps) => Step::Many(
                steps
                    .into_iter()
                    .map(|step| self.as_it_stands(step))
                    .collect(),
            ),
            // A move, a copy and a binning say where the files are and where
            // they came from, and neither half goes stale: running it forward
            // again is the same operation on the same paths.
            other => other,
        }
    }
}

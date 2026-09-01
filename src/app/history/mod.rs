//! Running the history: undo, redo, and going back to a chosen point.
//!
//! Kept out of [`super::cull`], where the undo used to live, because undo is
//! no longer a culling concern. It began as the safety net under the keys that
//! delete a photograph, which is why it was filed there; it now covers marks,
//! settings and where the program was pointed as well, and a file about taking
//! photographs out of the folder is the wrong place for it. That takes
//! `src/app/` one file past the fifteen this repository aims at, and shortens
//! `cull.rs` by more than it adds.
//!
//! What runs a deed is [`crate::history`]; what is here is the wiring — the
//! question a bulk run asks first, and putting the caches and the views right
//! afterwards.

use eframe::egui;

use crate::history::{Class, Deed, Done, NodeId, Way};

use super::App;

mod panel;
mod restore;
mod watch;

/// A run of the history the user has been asked about but has not answered.
///
/// The route is kept with the sentence so that what is carried out is exactly
/// what was described. Working it out again after the answer would be a second
/// chance to get a different one.
#[derive(Debug, Clone)]
pub struct Pending {
    /// What it is about to do, in a sentence.
    pub said: String,
    /// What the answer "yes" would run.
    pub route: Vec<(NodeId, Way)>,
}

impl App {
    /// Whether `Ctrl + Z` stops on deeds of this class.
    ///
    /// Everything is recorded whatever this says; the setting decides only
    /// where one press comes to rest.
    fn stops_on(&self, class: Class) -> bool {
        self.settings.history.undoes.get(class.name())
    }

    /// Takes back the last thing done.
    pub(super) fn undo(&mut self) {
        let route = self.history.plan_undo(|class| self.stops_on(class));
        self.consider(route, Way::Back);
    }

    /// Does again the thing that was last taken back.
    pub(super) fn redo(&mut self) {
        let route = self.history.plan_redo(|class| self.stops_on(class));
        self.consider(route, Way::Forward);
    }

    /// Asks first if the route would touch more than one file, then runs it.
    ///
    /// `cull.confirm.undo_several` is what decides, and until now it was a
    /// setting with a row in the registry that nothing ever read: the count
    /// was compared against one in the code and the user's answer ignored.
    fn consider(&mut self, route: Vec<(NodeId, Way)>, way: Way) {
        if route.is_empty() {
            self.notices.say(match way {
                Way::Back => "Nothing to undo",
                Way::Forward => "Nothing to do again",
            });
            return;
        }

        let files: usize = route
            .iter()
            .filter_map(|(id, _)| self.history.entry(*id))
            .map(|entry| entry.deed.files())
            .sum();

        if files > 1 && self.settings.cull.confirm.undo_several {
            self.pending_history = Some(Pending {
                said: self.describe_route(&route),
                route,
            });
            return;
        }

        self.run_history(route);
    }

    /// What a route would do, in one sentence.
    ///
    /// One deed says what it is; several say how many, because listing five
    /// sentences in a modal is a thing nobody reads.
    fn describe_route(&self, route: &[(NodeId, Way)]) -> String {
        let mut said = route
            .iter()
            .filter_map(|(id, way)| Some((self.history.entry(*id)?, *way)))
            .map(|(entry, way)| entry.deed.describe(way));

        let Some(first) = said.next() else {
            return "do nothing".to_string();
        };

        match said.count() {
            0 => first,
            more => format!("{first} — and {more} more"),
        }
    }

    /// Runs a route and puts the caches and the views right afterwards.
    fn run_history(&mut self, route: Vec<(NodeId, Way)>) {
        let said = self.describe_route(&route);
        let mut done = Done::default();

        for (id, way) in &route {
            let Some(entry) = self.history.entry(*id) else {
                continue;
            };

            match &entry.deed {
                Deed::Start => {}
                Deed::Files(step) => {
                    // Cloned because running it writes through `self`, and the
                    // step is borrowed out of the history it is stored in.
                    let step = step.clone();
                    let part = step.run(*way);

                    done.moved.extend(part.moved);
                    done.removed.extend(part.removed);
                    done.remarked.extend(part.remarked);
                    done.failed.extend(part.failed);
                }
                Deed::Changed(changes) => {
                    // Same reason, and going back means running them in the
                    // opposite order to the one they were recorded in.
                    let changes = changes.clone();

                    match way {
                        Way::Back => {
                            for change in changes.iter().rev() {
                                self.restore(change, Way::Back);
                            }
                        }
                        Way::Forward => {
                            for change in &changes {
                                self.restore(change, Way::Forward);
                            }
                        }
                    }
                }
            }
        }

        if let Some(landing) = self.history.landing(&route) {
            self.history.arrive(landing);
        }

        // The program has just been moved by the history rather than by the
        // user. Without this the next look would see the difference, file it
        // as a new deed, and undo would never let go of the end of the list.
        self.watching.resync();

        self.settle(done, said);
    }

    /// Tells the caches and the views what changed on disk.
    fn settle(&mut self, done: Done, said: String) {
        for problem in &done.failed {
            self.notices.say(format!("Could not do it: {problem}"));
        }

        for path in &done.remarked {
            // What is on the card has already been turned, and the sidecar has
            // just been written; the difference between the two is what the
            // card owes. The camera's own orientation is in both and cancels,
            // so this is the user's turn and nothing else.
            let was = self
                .annotations
                .peek(path)
                .map(|xmp| xmp.orientation)
                .unwrap_or_default();

            self.annotations.forget(path);
            self.refresh_mark(path);

            let now = self.annotations.get(path, None).orientation;
            let difference = was.inverse().then(now);

            if difference != crate::metadata::Orientation::Normal {
                self.image_view.turn_by(path, difference);
                self.grid_view.turn_by(path, difference);
            }
        }

        // Anything that came back or went away changes what the folder holds,
        // and the caches are keyed by position in it.
        if !done.moved.is_empty() || !done.removed.is_empty() {
            let base = self.base_path.clone();
            let showing = self.marked_path();
            self.open_directory(&base, showing.as_deref());
        }

        if done.failed.is_empty() && !said.is_empty() {
            self.notices.say(format!("Done: {said}"));
        }
    }

    /// Draws the question a bulk run of the history asks, and obeys it.
    pub(super) fn show_pending_history(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending_history.clone() else {
            return;
        };

        let mut answered = None;

        let shown = egui::Window::new("Undo")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!("This will {}.", pending.said));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Do it").clicked() {
                        answered = Some(true);
                    }
                    if ui.button("Leave it").clicked() {
                        answered = Some(false);
                    }
                });

                ui.add_space(6.0);
                ui.label(egui::RichText::new("Enter or Y to do it · Escape to leave it").weak());
            });

        crate::utils::in_front(ctx, shown.as_ref());

        let said_no = ctx.input_mut(|i| {
            let escaped = i.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
            escaped | i.consume_key(egui::Modifiers::NONE, egui::Key::N)
        });

        let said_yes = ctx.input_mut(|i| {
            let yes = i.consume_key(egui::Modifiers::NONE, egui::Key::Y);
            yes | i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
        });

        if said_no {
            answered = Some(false);
        } else if said_yes {
            answered = Some(true);
        }

        match answered {
            Some(true) => {
                self.pending_history = None;
                self.run_history(pending.route);
            }
            Some(false) => {
                self.pending_history = None;
            }
            None => {}
        }
    }
}

//! Everything around the picture: the side panel, the overlays, the folder
//! watcher and the benchmark.
//!
//! None of it is what the viewer is for, and all of it has to be drawn every
//! frame, which is why it is here rather than in the middle of the wiring.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use eframe::egui::{self, ViewportCommand};

use crate::crawler;
use crate::history::Panels;
use crate::ui::{navigator, tree};

use super::input::{self, Overlay};
use super::panels;
use super::watcher;
use super::App;

/// How much of the window a side panel may take before it is the window.
const MOST_OF_THE_WINDOW: f32 = 0.4;

impl App {
    pub(super) fn handle_watcher(&mut self) {
        if !self.watcher.is_active() {
            return;
        }

        // The events first, and the collection looked at only if there are
        // any. A watched folder is quiet almost every frame, and this used to
        // copy every path in it before finding that out — then answer each
        // question with a walk down the copy.
        let events = self.watcher.take_events();
        if events.is_empty() {
            return;
        }

        let known: HashSet<&Path> = self.paths.iter().map(PathBuf::as_path).collect();
        let changes = watcher::classify(events, |path| known.contains(path));

        if changes.is_empty() {
            return;
        }

        for path in &changes.modified {
            self.image_view.reload(path);
            self.grid_view.reload(path);
            self.annotations.forget(path);
            self.refresh_mark(path);
        }

        for path in &changes.removed {
            self.forget(path);
        }

        let moved = !changes.added.is_empty() || !changes.removed.is_empty();

        for path in changes.added {
            self.take_in(path);
        }

        // Once, after the batch, rather than once per file: what is on show is
        // a pass over the collection, and a tethered shoot arriving in bursts
        // would otherwise pay for it a frame at a time.
        if moved {
            self.apply_narrowing();
        }
    }

    /// Takes a photograph that has appeared into the open collection.
    ///
    /// At its sorted position, and without disturbing anything else. This used
    /// to read the folder again and hand both views a whole new collection,
    /// which threw away every decoded photograph and every thumbnail in it,
    /// put the cursor back on the newcomer, and cleared the selection — once
    /// per frame during a tethered shoot.
    fn take_in(&mut self, path: PathBuf) {
        if self.paths.contains(&path) {
            return;
        }

        // The other half of a raw+JPEG pair joins the photograph it belongs
        // to rather than the collection: a tethered shoot lands the two a
        // moment apart, and the second must not appear as a second frame.
        let prefer = self.settings.raw.pair_with_jpeg;
        if self.pairs.take_in(&path, &self.paths, prefer) {
            return;
        }

        let at = crawler::position_for(&self.paths, &path);

        self.add_mark(at, &path);
        self.paths.insert(at, path.clone());
        self.stacking.insert_shifting(at, &path);

        self.image_view.insert(at, path.clone());
        self.grid_view.insert(at, path);
    }

    /// Says which panels are on screen.
    ///
    /// The two menus that list the panels — the View menu on the bar and the
    /// Show submenu on the photograph — are drawn where neither these fields
    /// nor the configuration are in hand, so the answer is published once a
    /// frame rather than threaded through them. The same shape as
    /// `utils::set_in_front`, and for the same reason.
    ///
    /// It said which key shows and hides each of them too, until every menu in
    /// the program began naming its keys and the answer moved to the one table
    /// that holds all ninety ([`publish_keys`]). What is left is what only this
    /// type knows: which of the panels is up.
    ///
    /// [`publish_keys`]: Self::publish_keys
    pub(super) fn publish_panels(&self) {
        // Not collected first: the board writes over the rows it already has,
        // so publishing this costs nothing on a frame where no panel moved.
        crate::ui::panel::showing(crate::ui::panel::EVERY_PANEL.iter().map(|chrome| {
            crate::ui::panel::Showing {
                on: self.panel_is_showing(chrome.hide),
            }
        }));
    }

    /// Says what every command's keys read as, for the menus that name them.
    ///
    /// Read afresh rather than remembered. A copy kept beside the settings
    /// would be right until somebody rebound a key by a route that forgot to
    /// refresh it, and a menu naming a key that does nothing is worse than one
    /// naming none. The mode goes with it because a key is only a key where it
    /// is read: the same menu on the strip and in the contact sheet is two
    /// different sets of keys, and only one of them is true at a time.
    ///
    /// It is ninety short strings, written over rather than built again, once
    /// a frame — not a cost that multiplies by the size of a folder.
    pub(super) fn publish_keys(&self) {
        crate::ui::keys::publish(&self.settings, self.mode);
    }

    /// Which panels are on screen.
    ///
    /// One place, because there are four questions asked of these seven fields
    /// — the history's look, the history's restore, what a fullscreen mode
    /// puts away and what it puts back — and seven fields written out four
    /// times is four chances to leave one of them behind. It has happened
    /// once: the history panel was added to the half that reads and not to the
    /// half that writes, and could be opened and closed by every route except
    /// the one that undoes it.
    pub(super) fn panels(&self) -> Panels {
        Panels {
            menu: self.menu_visible,
            side: self.side_panel_visible,
            metrics: self.metrics_visible,
            tags: self.tag_panel_visible,
            filter: self.filter_visible,
            filmstrip: self.filmstrip_visible,
            history: self.history_panel_visible,
        }
    }

    /// Puts the panels the way this says.
    ///
    /// Taken apart field by field rather than read one at a time, because a
    /// struct pattern with no `..` is exhaustive: a panel added to [`Panels`]
    /// does not compile until it is put back here as well.
    pub(super) fn set_panels(&mut self, panels: Panels) {
        let Panels {
            menu,
            side,
            metrics,
            tags,
            filter,
            filmstrip,
            history,
        } = panels;

        self.menu_visible = menu;
        self.side_panel_visible = side;
        self.metrics_visible = metrics;
        self.tag_panel_visible = tags;
        self.filter_visible = filter;
        self.filmstrip_visible = filmstrip;
        self.history_panel_visible = history;
    }

    /// Which panels the user last chose to have on screen.
    ///
    /// The same distinction as `ImageView::opens` against `opens_at`: a
    /// fullscreen mode puts every panel away for its turn, and that is not a
    /// preference anybody expressed. So everything that *draws* asks
    /// [`Self::panels`], and the three places that write a panel to a file —
    /// the session's menu bar and the mirror's two — ask this one, or an
    /// evening watching a slideshow would end with a configuration saying the
    /// strip and the bar had been put away.
    pub(super) fn preferred_panels(&self) -> Panels {
        self.panels_before_fullscreen
            .unwrap_or_else(|| self.panels())
    }

    /// Sets which panels the user has chosen, wherever that answer is kept.
    pub(super) fn set_preferred_panels(&mut self, panels: Panels) {
        match &mut self.panels_before_fullscreen {
            Some(remembered) => *remembered = panels,
            None => self.set_panels(panels),
        }
    }

    /// Whether the panel that command puts away is on screen.
    fn panel_is_showing(&self, hide: Option<input::Command>) -> bool {
        match hide {
            // The status bar, which cannot be put away and is therefore not in
            // the list the menus draw.
            None => true,
            Some(input::Command::ToggleMenu) => self.menu_visible,
            Some(input::Command::ToggleMetrics) => self.metrics_visible,
            Some(input::Command::ToggleFilter) => self.filter_visible,
            Some(input::Command::ToggleSidePanel) => self.side_panel_visible,
            Some(input::Command::ToggleHistoryPanel) => self.history_panel_visible,
            Some(input::Command::ToggleTagPanel) => self.tag_panel_visible,
            Some(input::Command::ToggleFilmstrip) => self.filmstrip_visible,
            // A panel added to `EVERY_PANEL` without a line here would draw a
            // tick that never changes. The test at the foot of this file is
            // what says so, since the match cannot be asked without a window.
            Some(_) => true,
        }
    }

    pub(super) fn show_side_panel(&mut self, ctx: &egui::Context) {
        // An egui side panel sizes itself to its widest child unless it is
        // told not to, and the widest child here is the directory line. A
        // deep path used to take sixty per cent of the window and squeeze the
        // photograph into eleven per cent of it.
        let most = (ctx.content_rect().width() * MOST_OF_THE_WINDOW).max(220.);

        let mut asked: Option<&'static str> = None;
        let mut clip = false;
        let mut bind: Option<&'static str> = None;

        let (size, width, forced) = self.side_panel_size.asked_for();
        self.side_panel_size = size;

        let mut panel = egui::SidePanel::right("image_metadata")
            .resizable(true)
            .show_separator_line(false)
            .min_width(220.)
            .default_width(width)
            .max_width(most);

        // `default_width` is dead from the second frame on, so a width typed
        // into the settings window never reached this panel. For the one frame
        // after it moves by anything but a drag, the width is stated.
        if forced {
            panel = panel.exact_width(width);
        }

        let panel = panel.show_animated(ctx, self.side_panel_visible, |ui| {
            // A panel reports the rectangle its *contents* came to, not
            // the one the drag asked for, so a scroll area that shrinks to
            // its content reports the content's width and the next frame
            // is drawn at it — the edge springs back.
            ui.set_min_width(ui.available_width());

            egui::ScrollArea::vertical().show(ui, |ui| {
                // The same photograph the keyword panel is about. The two
                // used to read different things, so with both open they
                // described different photographs and neither said which.
                let showing = self.current_photograph();
                let metadata = showing
                    .as_deref()
                    .filter(|path| self.image_view.active_path().as_deref() == Some(path))
                    .and(self.image_view.active_metadata());

                asked = panels::metadata_panel(
                    ui,
                    metadata,
                    &self.config.metadata_tags,
                    !self.paths.is_empty(),
                )
                .or(asked);

                if let Some(found) = self.image_view.active_histogram() {
                    match crate::ui::histogram::show(ui, found, self.image_view.marking()) {
                        Some(crate::ui::histogram::Asked::Clipping) => clip = true,
                        Some(crate::ui::histogram::Asked::BindKey(path)) => bind = Some(path),
                        None => {}
                    }
                }
                ui.add_space(20.);
                ui.separator();
                asked = panels::cache_stats(ui, &self.image_view.stats(), &self.grid_view.stats())
                    .or(asked);
            });

            crate::ui::panel::menu(ui, &panels::METADATA_PANEL, |_| {});
        });

        // The dragged width, written back to the field the window reads. It
        // was a hardcoded `default_width(340.)`, so dragging the edge was a
        // gesture the viewer forgot on the way out.
        if let Some(response) = panel {
            let width = response.response.rect.width();
            if self.side_panel_visible && (width - self.config.side_panel_width).abs() > 1.0 {
                self.config.side_panel_width = width;
                self.settings.general.side_panel_width = width;
                self.side_panel_size = self.side_panel_size.dragged_to(width);
                self.save_settings();
            }
        }

        if clip {
            // The figure and the mask are the same question asked twice; the
            // mask lived in a different subsystem behind a key of its own.
            self.image_view.mark_clipping();
        }

        if let Some(path) = asked {
            // The reverse trip, from a readout to the setting behind it.
            self.open_settings_at(path);
        }

        // And from a readout to the key behind it, which is where the mask
        // lives: it is a key and a state and not a setting anywhere.
        if let Some(path) = bind {
            self.arm_key(path);
        }
    }

    /// Draws the navigator and directory tree overlays.
    pub(super) fn show_overlays(&mut self, ctx: &egui::Context) {
        match self.overlay {
            Some(Overlay::Navigator) => {
                if navigator::ui(&mut self.navigator_path, ctx) {
                    let path = PathBuf::from(self.navigator_path.clone());
                    input::close(&mut self.overlay);
                    self.open_directory(&path, None);
                }
            }
            Some(Overlay::DirectoryTree) => {
                let opened = self
                    .image_view
                    .active_path()
                    .unwrap_or_else(|| self.base_path.clone());

                if let Some(path) = tree::ui(&opened.to_string_lossy(), ctx) {
                    input::close(&mut self.overlay);
                    self.open_directory(&path, None);
                }
            }
            None => {}
        }
    }

    /// Walks the folder as fast as it will go, then reports and quits.
    pub(super) fn run_benchmark(&mut self, ctx: &egui::Context) {
        let Some(benchmark) = &mut self.benchmark else {
            return;
        };

        // Nothing is idle during a benchmark, so the next frame starts at once.
        ctx.request_repaint();

        let before = self.image_view.selected_index();
        self.image_view.next_image();
        let moved = self.image_view.selected_index() != before;

        if benchmark.frame(self.perf_metrics.last_frame(), moved) {
            return;
        }

        let report = benchmark.report();

        // Put down before the report, because asking to close still leaves a
        // frame or two to draw and the run used to report itself on each of
        // them.
        self.benchmark = None;

        report.log();
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}

#[cfg(test)]
mod tests {
    use super::input::Command;

    /// What `panel_is_showing` answers, written down because the match cannot
    /// be asked without a window.
    const READ_BACK: &[Command] = &[
        Command::ToggleMenu,
        Command::ToggleMetrics,
        Command::ToggleFilter,
        Command::ToggleSidePanel,
        Command::ToggleHistoryPanel,
        Command::ToggleTagPanel,
        Command::ToggleFilmstrip,
    ];

    /// Every panel that can be put away is one `panel_is_showing` reads back.
    ///
    /// A panel added to `EVERY_PANEL` fails this rather than drawing a tick in
    /// the View menu that never changes whatever is done to it.
    #[test]
    fn every_panel_that_can_be_put_away_is_read_back() {
        for chrome in crate::ui::panel::EVERY_PANEL {
            let Some(hide) = chrome.hide else {
                continue;
            };

            assert!(
                READ_BACK.contains(&hide),
                "{} is put away by {hide:?}, which `panel_is_showing` does not read back",
                chrome.subject.said()
            );
        }
    }

    /// And every one of them is a field a fullscreen mode can put away.
    ///
    /// [`super::App::set_panels`] is exhaustive over [`Panels`], so nothing
    /// there can be left behind; what can is a panel given a flag on `App` and
    /// a row in the list above with no field in `Panels` at all — which is a
    /// panel a slideshow would leave on the screen, that the history would
    /// never put back, and that no route could write to a file.
    #[test]
    fn every_panel_read_back_is_one_a_fullscreen_mode_puts_away() {
        assert_eq!(READ_BACK.len(), crate::history::Panels::EACH.len());
    }
}

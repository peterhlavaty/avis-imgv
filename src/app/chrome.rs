//! Everything around the picture: the side panel, the overlays, the folder
//! watcher and the benchmark.
//!
//! None of it is what the viewer is for, and all of it has to be drawn every
//! frame, which is why it is here rather than in the middle of the wiring.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use eframe::egui::{self, ViewportCommand};

use crate::crawler;
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

        self.image_view.insert(at, path.clone());
        self.grid_view.insert(at, path);
    }

    pub(super) fn show_side_panel(&mut self, ctx: &egui::Context) {
        // An egui side panel sizes itself to its widest child unless it is
        // told not to, and the widest child here is the directory line. A
        // deep path used to take sixty per cent of the window and squeeze the
        // photograph into eleven per cent of it.
        let most = (ctx.content_rect().width() * MOST_OF_THE_WINDOW).max(220.);

        egui::SidePanel::right("image_metadata")
            .resizable(true)
            .show_separator_line(false)
            .min_width(220.)
            .default_width(340.)
            .max_width(most)
            .show_animated(ctx, self.side_panel_visible, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    panels::metadata_panel(
                        ui,
                        self.image_view.active_metadata(),
                        &self.config.metadata_tags,
                    );
                    ui.add_space(20.);
                    ui.separator();
                    panels::cache_stats(ui, &self.image_view.stats(), &self.grid_view.stats());
                });
            });
    }

    /// Draws the navigator and directory tree overlays.
    pub(super) fn show_overlays(&mut self, ctx: &egui::Context) {
        match self.overlay {
            Some(Overlay::Navigator) => {
                if navigator::ui(&mut self.navigator_path, ctx) {
                    let path = PathBuf::from(self.navigator_path.clone());
                    input::close(ctx, &mut self.overlay);
                    self.open_directory(&path, None);
                }
            }
            Some(Overlay::DirectoryTree) => {
                let opened = self
                    .image_view
                    .active_path()
                    .unwrap_or_else(|| self.base_path.clone());

                if let Some(path) = tree::ui(&opened.to_string_lossy(), ctx) {
                    input::close(ctx, &mut self.overlay);
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

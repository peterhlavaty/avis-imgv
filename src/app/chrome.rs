//! Everything around the picture: the side panel, the overlays, the folder
//! watcher and the benchmark.
//!
//! None of it is what the viewer is for, and all of it has to be drawn every
//! frame, which is why it is here rather than in the middle of the wiring.

use std::path::PathBuf;

use eframe::egui::{self, ViewportCommand};

use crate::ui::{navigator, tree};

use super::input::{self, Overlay};
use super::panels;
use super::App;

impl App {
    pub(super) fn handle_watcher(&mut self) {
        if !self.watcher.is_active() {
            return;
        }

        let known: Vec<PathBuf> = self.paths.clone();
        let changes = self
            .watcher
            .take_changes(|path| known.iter().any(|candidate| candidate == path));

        if changes.is_empty() {
            return;
        }

        for path in &changes.modified {
            self.image_view.reload(path);
            self.grid_view.reload(path);
            self.annotations.forget(path);
        }

        if !changes.added.is_empty() {
            let newest = changes.added.last().cloned();
            self.paths.extend(changes.added);
            self.paths.sort();

            let paths = self.paths.clone();
            self.image_view.set_images(paths.clone(), newest.as_deref());
            self.grid_view.set_images(paths);
        }
    }

    pub(super) fn show_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("image_metadata")
            .resizable(true)
            .show_separator_line(false)
            .min_width(220.)
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

//! The folder modes: renaming a shoot, and correcting a camera clock.
//!
//! Both are the same shape. Narrow the folder down, put it in an order, look
//! at exactly what would happen to every file, and only then apply it. What
//! differs is the middle panel, which is why the two live beside each other
//! here rather than in two separate views.

mod controls;
mod group;
mod rename;
mod table;
mod thumbnails;
mod timeshift;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use eframe::egui;

use crate::app::mode::Mode;
use crate::organize::group::{Group, Settings as Grouping};
use crate::organize::{self, rename as renaming, timeshift as shifting};
use crate::organize::{Direction, Entry, Filter, Scan, SortKey};

/// What the view did, so the application can pick the folder up again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Done {
    /// Files were renamed, so every path the application holds is stale.
    Renamed,
    /// Timestamps moved, so the metadata is stale but the paths are not.
    Shifted,
}

pub struct OrganizeView {
    /// Every file in the open collection, filled in by the scan.
    all: Vec<Entry>,
    scan: Option<Scan>,
    /// The narrowed, ordered list the job applies to. Derived from `all`.
    selection: Vec<Entry>,
    /// Set whenever something that changes `selection` is touched.
    stale: bool,

    sort_key: SortKey,
    /// The tag typed into the "other metadata" box, kept even while a
    /// different key is chosen so switching back does not lose it.
    sort_tag: String,
    direction: Direction,
    filter: Filter,
    filter_open: bool,

    rename: renaming::Options,
    offset: shifting::Offset,
    /// Which timestamps to move. Empty means all of them.
    chosen_fields: BTreeSet<String>,

    grouping: Grouping,
    /// Height of the thumbnails in the group panel, in points. Zero shows the
    /// names alone.
    thumbnail_height: f32,
    thumbnails: thumbnails::Thumbnails,
    /// The proposed groups, as the user has since edited them.
    groups: Vec<Group>,
    /// The frames that belong to none of them.
    loose: Vec<Entry>,
    /// Set when the selection or the grouping settings have moved on, so the
    /// groups are read again — but not on every frame, which would throw away
    /// every correction the user had made.
    groups_stale: bool,

    /// The last thing that happened, shown under the buttons.
    status: String,
}

impl Default for OrganizeView {
    fn default() -> Self {
        Self::new()
    }
}

impl OrganizeView {
    pub fn new() -> OrganizeView {
        OrganizeView {
            all: Vec::new(),
            scan: None,
            selection: Vec::new(),
            stale: true,
            sort_key: SortKey::default(),
            sort_tag: String::new(),
            direction: Direction::default(),
            filter: Filter::new(),
            filter_open: false,
            rename: renaming::Options::default(),
            offset: shifting::Offset::default(),
            chosen_fields: BTreeSet::new(),
            grouping: Grouping::default(),
            thumbnail_height: thumbnails::SIZES[2].1,
            thumbnails: thumbnails::Thumbnails::default(),
            groups: Vec::new(),
            loose: Vec::new(),
            groups_stale: true,
            status: String::new(),
        }
    }

    /// Opens a collection, starting the sweep that reads it.
    pub fn set_images(&mut self, paths: Vec<PathBuf>) {
        self.all = organize::entries(&paths);
        // Replacing it stops the previous one.
        self.scan = Some(Scan::start(paths));
        self.status.clear();
        self.stale = true;
        self.groups_stale = true;
        self.thumbnails.clear();
    }

    /// Whether the folder has already been read into this view.
    pub fn holds(&self, paths: &[PathBuf]) -> bool {
        self.all.len() == paths.len()
            && self
                .all
                .iter()
                .zip(paths)
                .all(|(entry, path)| &entry.path == path)
    }

    /// Draws the mode, returning what it did to the folder.
    pub fn ui(&mut self, ctx: &egui::Context, mode: Mode) -> Option<Done> {
        if self.collect_scan() {
            ctx.request_repaint();
        }

        self.refresh();

        let mut done = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading(mode.label());
            self.show_progress(ui);
            ui.add_space(8.0);

            controls::show(ui, self);
            ui.separator();

            done = match mode {
                Mode::Rename => rename::show(ui, self),
                Mode::TimeShift => timeshift::show(ui, self),
                Mode::Group => group::show(ui, self),
                // The application only reaches here in a folder mode.
                _ => None,
            };
        });

        done
    }

    /// Takes in whatever the sweep has read.
    fn collect_scan(&mut self) -> bool {
        let Some(scan) = &mut self.scan else {
            return false;
        };

        let arrived = scan.collect_into(&mut self.all);
        if arrived {
            self.stale = true;
            self.groups_stale = true;
        }

        if scan.is_finished() && !arrived {
            self.scan = None;
        }

        arrived
    }

    /// Rebuilds the narrowed, ordered list when something has changed it.
    fn refresh(&mut self) {
        if !self.stale {
            return;
        }

        let key = self.key();

        self.selection = self.all.clone();
        self.filter.apply(&mut self.selection);
        organize::sort::sort(&mut self.selection, &key, self.direction);

        self.stale = false;
    }

    /// Reads the selection into groups again, if something has moved.
    ///
    /// Only when it has: the groups carry the user's corrections, and redoing
    /// the detection every frame would undo them as fast as they were made.
    fn regroup_if_stale(&mut self) {
        if !self.groups_stale {
            return;
        }

        let (groups, loose) = group::regrouped(&self.selection, &self.grouping);

        self.groups = groups;
        self.loose = loose;
        self.groups_stale = false;
    }

    /// The folder the pictures are in, which is where new folders are made.
    fn folder(&self) -> PathBuf {
        self.all
            .first()
            .and_then(|entry| entry.path.parent())
            .map(Path::to_path_buf)
            .unwrap_or_default()
    }

    /// The sort key as chosen, with the typed tag folded in.
    fn key(&self) -> SortKey {
        match &self.sort_key {
            SortKey::Metadata(_) => SortKey::Metadata(self.sort_tag.trim().to_string()),
            other => other.clone(),
        }
    }

    fn show_progress(&self, ui: &mut egui::Ui) {
        let Some(scan) = &self.scan else {
            return;
        };

        let (done, total) = scan.progress();
        if total == 0 || done >= total {
            return;
        }

        ui.horizontal(|ui| {
            ui.add(egui::Spinner::new().size(14.0));
            ui.label(format!("Reading the folder… {done} of {total}"));
        });
    }

    /// How many files the job applies to, and how many there are.
    fn counts(&self) -> (usize, usize) {
        (self.selection.len(), self.all.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view_over(names: &[&str]) -> OrganizeView {
        let mut view = OrganizeView::new();
        view.all = names
            .iter()
            .map(|name| Entry::new(PathBuf::from("/photos").join(name)))
            .collect();
        view.stale = true;

        view
    }

    #[test]
    fn a_fresh_view_narrows_to_nothing_being_narrowed() {
        let mut view = view_over(&["b.jpg", "a.jpg"]);
        view.refresh();

        assert_eq!(view.counts(), (2, 2));
        assert_eq!(view.selection[0].name(), "a.jpg", "and it is sorted");
    }

    #[test]
    fn a_filter_narrows_the_selection_without_losing_the_folder() {
        let mut view = view_over(&["a.jpg", "b.png"]);
        view.filter.extensions = "jpg".into();
        view.stale = true;
        view.refresh();

        assert_eq!(view.counts(), (1, 2));
    }

    #[test]
    fn the_typed_tag_only_applies_when_the_key_is_a_tag() {
        let mut view = view_over(&["a.jpg"]);
        view.sort_tag = "ISO".into();

        assert_eq!(view.key(), SortKey::Name);

        view.sort_key = SortKey::Metadata(String::new());
        assert_eq!(view.key(), SortKey::Metadata("ISO".into()));
    }

    #[test]
    fn a_view_knows_whether_it_already_holds_a_folder() {
        let paths = vec![
            PathBuf::from("/photos/a.jpg"),
            PathBuf::from("/photos/b.jpg"),
        ];
        let view = view_over(&["a.jpg", "b.jpg"]);

        assert!(view.holds(&paths));
        assert!(!view.holds(&paths[..1]));
        assert!(!view_over(&["c.jpg", "d.jpg"]).holds(&paths));
    }
}

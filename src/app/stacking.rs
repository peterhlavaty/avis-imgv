//! Running the group detector over the folder that is open.
//!
//! The detector needs what the files say about themselves — when each frame
//! was taken, and roughly what it looks like — and reading that for a folder
//! of two thousand takes a couple of seconds. So it is not done at all until
//! somebody asks for stacks, and when they do it happens on the scan the
//! organiser already uses: results arrive in batches, and the sheet folds up
//! as they land rather than after a wait in front of a still screen.
//!
//! What the user has opened stays open across a re-detection. A batch arriving
//! every thirty milliseconds must not close a stack somebody has just opened,
//! and the tolerance slider is meant to be dragged — which redetects on every
//! step.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::organize::group::{self, Settings};
use crate::organize::{Entry, Scan};
use crate::view::stacks::Stacks;
use crate::view::visible::Visible;

#[derive(Default)]
pub struct Stacking {
    /// Whether the folder is being shown stacked at all.
    on: bool,
    /// How the detector reads the folder. Changing any of it redetects.
    settings: Settings,
    /// One per photograph in the collection, filled in by the scan.
    entries: Vec<Entry>,
    scan: Option<Scan>,
    stacks: Stacks,
    /// The first frame of every stack the user has opened, so a redetection
    /// does not shut them all again.
    opened: HashSet<usize>,
}

impl Stacking {
    /// What the configuration says a run of frames is, and whether a folder
    /// opens with them folded.
    pub fn of(config: &crate::config::GroupConfig, on: bool) -> Stacking {
        Stacking {
            on,
            settings: Settings::of(config),
            ..Stacking::default()
        }
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    pub fn stacks(&self) -> &Stacks {
        &self.stacks
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    /// How far the reading has got, while it is still going.
    pub fn progress(&self) -> Option<(usize, usize)> {
        let scan = self.scan.as_ref()?;
        let (done, total) = scan.progress();

        (done < total).then_some((done, total))
    }

    /// Starts reading `paths`, and shows the folder stacked once it can.
    pub fn turn_on(&mut self, paths: &[PathBuf]) {
        self.on = true;
        self.opened.clear();
        self.stacks = Stacks::default();
        self.entries = crate::organize::entries(paths);
        self.scan = Some(Scan::start(paths.to_vec()));
    }

    /// Puts the folder back the way it was, and forgets what was read.
    ///
    /// The scan is dropped, which stops it: turning stacking off in the middle
    /// of reading a large folder should give the cores back at once.
    pub fn turn_off(&mut self) {
        self.on = false;
        self.scan = None;
        self.entries = Vec::new();
        self.stacks = Stacks::default();
        self.opened.clear();
    }

    /// Follows the collection when it changes underneath.
    ///
    /// A different folder is a different set of runs, so the reading starts
    /// again; a folder that is not being stacked stays unread.
    pub fn reopen(&mut self, paths: &[PathBuf]) {
        if self.on {
            self.turn_on(paths);
        }
    }

    /// Takes whatever the scan has read. Returns whether the stacks changed.
    pub fn poll(&mut self, paths: &[PathBuf]) -> bool {
        if !self.on {
            return false;
        }

        let Some(scan) = self.scan.as_mut() else {
            return false;
        };

        if !scan.collect_into(&mut self.entries) {
            return false;
        }

        if scan.is_finished() {
            self.scan = None;
        }

        self.detect(paths);
        true
    }

    /// Reads the folder into runs again, keeping what the user has opened.
    pub fn detect(&mut self, paths: &[PathBuf]) {
        let groups = group::detect(&self.entries, &self.settings);
        let positions = crate::view::stacks::positions(paths);

        let mut stacks = Stacks::of_groups(&groups, |path| positions.get(path).copied(), true);
        stacks.open(&self.opened);

        self.stacks = stacks;
    }

    /// Reads the folder differently: a longer gap, or a looser idea of what
    /// counts as the same scene.
    pub fn retune(&mut self, settings: Settings, paths: &[PathBuf]) -> bool {
        if settings == self.settings {
            return false;
        }

        self.settings = settings;
        self.detect(paths);
        true
    }

    /// Opens or closes the stack a photograph is in.
    pub fn toggle(&mut self, index: usize) -> bool {
        if !self.stacks.toggle(index) {
            return false;
        }

        self.opened = self.stacks.opened();
        true
    }

    /// Closes every stack, or opens every one.
    pub fn set_all(&mut self, collapsed: bool) {
        self.stacks.set_all(collapsed);
        self.opened = self.stacks.opened();
    }

    /// Changes which frame stands for the stack a photograph is in.
    pub fn step_standing(&mut self, index: usize, forward: bool) -> Option<usize> {
        self.stacks.step_standing(index, forward)
    }

    /// What to show, given what the filter left.
    pub fn fold(&self, visible: Visible, total: usize) -> Visible {
        if !self.on {
            return visible;
        }

        self.stacks.fold(&visible, total)
    }
}

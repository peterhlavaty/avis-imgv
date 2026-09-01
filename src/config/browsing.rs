//! What a folder opens as, and what counts as one run of frames.
//!
//! Two sections that did not exist. Everything in them is state the program
//! already keeps and throws away on exit — the order, the filter, whether the
//! folder was stacked, how far apart two frames may be and still be one burst —
//! or a constant nobody could reach. What is being added is mostly persistence,
//! not choice.

use serde::{Deserialize, Serialize};

use crate::view::narrow::{FlagRule, SortBy};

/// How a folder is ordered and narrowed when it is opened.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct BrowsingConfig {
    /// What a folder is ordered by.
    #[serde(default)]
    pub sort: SortBy,
    /// Whether that order runs backwards.
    #[serde(default)]
    pub descending: bool,
    /// Which flag a photograph has to carry to be shown.
    #[serde(default)]
    pub flag: FlagRule,
    /// Fewest stars shown.
    #[serde(default)]
    pub min_stars: u8,
    /// Most stars shown.
    #[serde(default = "default_max_stars")]
    pub max_stars: u8,
    /// Which colour label has to be on it: `any`, `none`, or a colour.
    #[serde(default = "default_label_rule")]
    pub label: String,
    /// Whether the rules survive opening another folder.
    ///
    /// They do today, and nothing resets them, which is right for somebody
    /// walking a card of shoots with "three stars and better" up and wrong for
    /// somebody who set a rule once and forgot.
    #[serde(default = "yes")]
    pub filter_follows_folder: bool,
    /// Whether a folder opens with its bursts folded into one cell each.
    #[serde(default)]
    pub stack_by_default: bool,
}

fn default_max_stars() -> u8 {
    crate::metadata::xmp::MAX_RATING as u8
}

fn default_label_rule() -> String {
    "any".to_string()
}

fn yes() -> bool {
    true
}

impl Default for BrowsingConfig {
    fn default() -> Self {
        BrowsingConfig {
            sort: SortBy::default(),
            descending: false,
            flag: FlagRule::default(),
            min_stars: 0,
            max_stars: default_max_stars(),
            label: default_label_rule(),
            filter_follows_folder: true,
            stack_by_default: false,
        }
    }
}

/// What counts as one run of frames.
///
/// One set of thresholds, read by the contact sheet's stacking and by the
/// organiser's Group shots. There used to be two, tuned by two control sets
/// that did not span the same ranges — the filter bar allowed a gap of 1 to 600
/// seconds and a tolerance of 0 to 32, the group panel 1 to 3600 and 0 to 64 —
/// so there were answers one surface could express and the other could not.
/// Two answers to "is this one burst?" is a defect whether or not anybody
/// navigates between them.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
pub struct GroupConfig {
    /// The longest pause between two frames that is still one run, in seconds.
    #[serde(default = "default_max_gap")]
    pub max_gap: f32,
    /// How different two thumbnails may look and still count as the same view.
    /// Zero is identical; sixty-four accepts anything.
    #[serde(default = "default_tolerance")]
    pub tolerance: u32,
    /// Fewest frames that make a run rather than a coincidence.
    #[serde(default = "default_min_frames")]
    pub min_frames: usize,
}

fn default_max_gap() -> f32 {
    60.0
}

fn default_tolerance() -> u32 {
    12
}

fn default_min_frames() -> usize {
    2
}

impl Default for GroupConfig {
    fn default() -> Self {
        GroupConfig {
            max_gap: default_max_gap(),
            tolerance: default_tolerance(),
            min_frames: default_min_frames(),
        }
    }
}

/// Which panels a launch starts with.
///
/// One key rather than five, drawn as five ticks: they are one decision about
/// what the window looks like when it opens.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(default)]
pub struct PanelsAtStart {
    pub menu: bool,
    /// The metadata and cache readout down the side.
    pub side_panel: bool,
    /// The stars and keywords panel.
    pub tag_panel: bool,
    /// The strip of thumbnails under the photograph.
    pub filmstrip: bool,
    /// The list of what has been done.
    #[serde(default)]
    pub history: bool,
}

impl PanelsAtStart {
    /// Every panel, by the name the file uses.
    pub const NAMES: &'static [&'static str] =
        &["menu", "side_panel", "tag_panel", "filmstrip", "history"];

    pub fn get(&self, name: &str) -> bool {
        match name {
            "menu" => self.menu,
            "side_panel" => self.side_panel,
            "tag_panel" => self.tag_panel,
            "filmstrip" => self.filmstrip,
            "history" => self.history,
            _ => false,
        }
    }

    pub fn set(&mut self, name: &str, on: bool) {
        match name {
            "menu" => self.menu = on,
            "side_panel" => self.side_panel = on,
            "tag_panel" => self.tag_panel = on,
            "filmstrip" => self.filmstrip = on,
            "history" => self.history = on,
            _ => {}
        }
    }
}

/// Which irreversible-looking things ask first.
///
/// Only the ones the journal covers can be switched off. Deleting for good and
/// putting the keyboard back have no inverse anywhere, so both always ask and
/// neither is a setting: a confirmation is not a substitute for reversibility,
/// because people click through dialogues by reflex.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct Confirmations {
    /// Moving more than one photograph to the bin.
    #[serde(default = "yes")]
    pub bin_several: bool,
    /// Emptying the rejects.
    #[serde(default = "yes")]
    pub empty_rejects: bool,
    /// Undoing a step that touched more than one file.
    #[serde(default = "yes")]
    pub undo_several: bool,
}

impl Confirmations {
    pub const NAMES: &'static [&'static str] = &["bin_several", "empty_rejects", "undo_several"];

    pub fn get(&self, name: &str) -> bool {
        match name {
            "bin_several" => self.bin_several,
            "empty_rejects" => self.empty_rejects,
            "undo_several" => self.undo_several,
            _ => false,
        }
    }

    pub fn set(&mut self, name: &str, on: bool) {
        match name {
            "bin_several" => self.bin_several = on,
            "empty_rejects" => self.empty_rejects = on,
            "undo_several" => self.undo_several = on,
            _ => {}
        }
    }
}

impl Default for Confirmations {
    fn default() -> Self {
        Confirmations {
            bin_several: true,
            empty_rejects: true,
            undo_several: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two surfaces that detect runs read one set of thresholds now.
    #[test]
    fn the_grouping_thresholds_have_one_home() {
        let group = GroupConfig::default();

        assert_eq!(group.max_gap, 60.0);
        assert_eq!(group.tolerance, 12);
        assert_eq!(group.min_frames, 2);
    }

    /// A folder opens showing everything, in name order.
    #[test]
    fn a_folder_opens_showing_everything() {
        let browsing = BrowsingConfig::default();

        assert_eq!(browsing.min_stars, 0);
        assert_eq!(browsing.max_stars, crate::metadata::xmp::MAX_RATING as u8);
        assert_eq!(browsing.flag, FlagRule::Any);
        assert_eq!(browsing.label, "any");
        assert!(!browsing.descending);
    }

    #[test]
    fn every_panel_can_be_read_and_written_by_name() {
        let mut panels = PanelsAtStart::default();

        for name in PanelsAtStart::NAMES {
            assert!(!panels.get(name));
            panels.set(name, true);
            assert!(panels.get(name), "{name} did not stay set");
        }
    }

    /// All three start on, and an unknown name changes nothing.
    #[test]
    fn every_confirmation_starts_on() {
        let mut confirm = Confirmations::default();

        for name in Confirmations::NAMES {
            assert!(confirm.get(name), "{name} starts off");
            confirm.set(name, false);
            assert!(!confirm.get(name));
        }

        confirm.set("nonsense", true);
        assert!(!confirm.bin_several);
    }
}

/// What the second button offers.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct MenuConfig {
    /// Whether the built-in menus carry their settings rows.
    ///
    /// The whole of the configurability offered for the built-in rows, and the
    /// reason a menu editor is not needed: turning it off leaves the verbs,
    /// your own entries, the copy group and the last row, so nothing becomes
    /// unreachable. A person who wants a four-row menu and a person who wants a
    /// nine-row one are both being reasonable, which is the test a new field
    /// has to pass.
    #[serde(default = "yes")]
    pub settings_rows: bool,
}

impl Default for MenuConfig {
    fn default() -> Self {
        MenuConfig {
            settings_rows: true,
        }
    }
}

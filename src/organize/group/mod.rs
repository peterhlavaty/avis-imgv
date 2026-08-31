//! Finding the shots that belong together.
//!
//! A photographer rarely takes one frame of anything worth taking. A bracket
//! for a high dynamic range merge is three or five frames of the same view at
//! different exposures; a focus stack is a dozen at different distances; a
//! timelapse is hundreds at a steady interval; and a burst is however many it
//! took to catch the moment. All of them arrive in one folder, interleaved
//! with the single frames, and all of them are better dealt with together.
//!
//! What separates them is visible in what the files already say. A run of
//! frames close in time and showing the same scene is a group; what kind of
//! group it is follows from what varies across it — the exposure for a
//! bracket, the focus distance for a stack, nothing at all but a steady
//! interval for a timelapse.
//!
//! The answer is a proposal, not a verdict: every group can be retyped,
//! dissolved, or have frames taken out of it before anything moves.

pub mod classify;
mod split;

pub use classify::Kind;

use crate::metadata::datetime::Timestamp;

use super::sharpness;
use super::Entry;

/// How the folder should be read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The longest gap between two frames that can still be one group.
    ///
    /// A minute covers a timelapse at a leisurely interval and a bracket taken
    /// by hand, and is short enough that two separate subjects photographed
    /// one after the other do not run together.
    pub max_gap: f64,
    /// How different two thumbnails may be and still count as the same scene.
    /// Sixty-four is every bit, which is no test at all.
    pub tolerance: u32,
    /// Groups smaller than this are left alone as single frames.
    pub min_frames: usize,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            max_gap: 60.0,
            tolerance: 12,
            min_frames: 2,
        }
    }
}

/// A run of frames that belong together, and what kind of run it is.
#[derive(Debug, Clone)]
pub struct Group {
    pub kind: Kind,
    /// The files, in the order they were taken.
    pub members: Vec<Entry>,
    /// What the detector thought before the user touched anything, so a group
    /// that was retyped can say so.
    pub detected: Kind,
}

impl Group {
    pub fn new(kind: Kind, members: Vec<Entry>) -> Group {
        Group {
            kind,
            members,
            detected: kind,
        }
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Whether the user has overruled the detector.
    pub fn was_retyped(&self) -> bool {
        self.kind != self.detected
    }

    /// When the first frame was taken.
    pub fn started(&self) -> Option<Timestamp> {
        self.members.first().and_then(Entry::captured)
    }

    /// How long the whole run took, in seconds.
    pub fn span(&self) -> Option<f64> {
        let first = self.members.first().and_then(Entry::captured)?;
        let last = self.members.last().and_then(Entry::captured)?;

        Some((last.to_seconds() - first.to_seconds()) as f64)
    }

    /// Which frame of this group looked sharpest.
    ///
    /// The one question a burst is really asking, and the one a contact sheet
    /// is worst at answering: at thumbnail size five frames of the same thing
    /// all look acceptable. Comparing them is what the measure is actually
    /// good for — they are the same scene at the same size a second apart, so
    /// the only thing that differs is the focus.
    ///
    /// `None` when nothing in the group could be measured, rather than the
    /// first frame by default: offering an arbitrary frame as "the sharpest"
    /// would be worse than offering none.
    /// On a tie the earlier frame wins, which is written out rather than left
    /// to whichever way `max_by` happens to break one: two frames that measure
    /// the same are the same, and the one taken first is the one a person
    /// would have been offered anyway.
    pub fn sharpest(&self) -> Option<usize> {
        let mut best: Option<(usize, sharpness::Sharpness)> = None;

        for (at, entry) in self.members.iter().enumerate() {
            let Some(found) = entry.sharpness else {
                continue;
            };

            if best.is_none_or(|(_, sharpest)| found.value() > sharpest.value()) {
                best = Some((at, found));
            }
        }

        best.map(|(at, _)| at)
    }

    /// A sentence for the group's header.
    pub fn describe(&self) -> String {
        let frames = format!("{} frames", self.len());

        match self.span() {
            Some(span) if span >= 1.0 => format!("{frames} over {}", duration(span)),
            _ => format!("{frames}, all at once"),
        }
    }
}

/// Reads a folder into the groups it contains.
///
/// `entries` is taken in whatever order it arrives and put in the order the
/// frames were taken, because that is the only order the question makes sense
/// in. Files with no capture time cannot be placed in a sequence and are left
/// out; so is every run too short to be worth calling a group.
pub fn detect(entries: &[Entry], settings: &Settings) -> Vec<Group> {
    let mut dated: Vec<Entry> = entries
        .iter()
        .filter(|entry| entry.captured().is_some())
        .cloned()
        .collect();

    dated.sort_by_key(|entry| {
        (
            entry.captured().map(Timestamp::to_seconds).unwrap_or(0),
            entry.name().to_string(),
        )
    });

    split::runs(&dated, settings)
        .into_iter()
        .filter(|run| run.len() >= settings.min_frames.max(2))
        .map(|run| Group::new(classify::kind(&run), run))
        .collect()
}

/// The frames that ended up in no group, in the order they were taken.
pub fn ungrouped(entries: &[Entry], groups: &[Group]) -> Vec<Entry> {
    let taken: Vec<&std::path::Path> = groups
        .iter()
        .flat_map(|group| group.members.iter().map(|entry| entry.path.as_path()))
        .collect();

    entries
        .iter()
        .filter(|entry| !taken.contains(&entry.path.as_path()))
        .cloned()
        .collect()
}

/// A span of seconds, as a person would say it.
fn duration(seconds: f64) -> String {
    let seconds = seconds.round() as i64;

    match seconds {
        ..=59 => format!("{seconds} s"),
        60..=3599 => format!("{} min {} s", seconds / 60, seconds % 60),
        _ => format!("{} h {} min", seconds / 3600, seconds % 3600 / 60),
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::super::similarity::Fingerprint;
    use super::super::test_support::entry;
    use super::super::{Entry, CAPTURE_TAG};
    use crate::metadata::datetime::Timestamp;

    /// A frame taken `at` seconds past a fixed midnight, looking like `scene`.
    ///
    /// `scene` stands in for the fingerprint: frames of the same scene are
    /// given the same one, and a different scene a distant one.
    pub fn frame(name: &str, at: i64, scene: u64) -> Entry {
        let taken = Timestamp::from_seconds(
            Timestamp::parse("2024:11:06 12:00:00")
                .unwrap()
                .to_seconds()
                + at,
        );

        let mut entry = entry(name, 0, &[(CAPTURE_TAG, &taken.to_exif())]);
        entry.fingerprint = Some(Fingerprint::from_bits(scene));

        entry
    }

    /// Adds a metadata tag to a frame.
    pub fn tagged(mut entry: Entry, pairs: &[(&str, &str)]) -> Entry {
        if let Some(metadata) = entry.metadata.as_mut() {
            for (key, value) in pairs {
                metadata.tags.insert(key.to_string(), value.to_string());
            }
        }

        entry
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::frame;
    use super::*;

    fn names(group: &Group) -> Vec<&str> {
        group.members.iter().map(Entry::name).collect()
    }

    /// The question a burst is really asking.
    #[test]
    fn the_sharpest_frame_of_a_group_is_found() {
        let mut members = vec![
            frame("a.jpg", 0, 1),
            frame("b.jpg", 1, 1),
            frame("c.jpg", 2, 1),
        ];

        members[0].sharpness = Some(sharpness::Sharpness::default());
        members[2].sharpness = Some(sharpness::Sharpness::default());

        // Nothing measured yet on the middle one, and the two that were are
        // equal, so the first of them wins.
        let group = Group::new(Kind::Series, members.clone());
        assert_eq!(group.sharpest(), Some(0));
    }

    /// Nothing measurable is `None` rather than the first frame: offering an
    /// arbitrary frame as the sharpest is worse than offering none.
    #[test]
    fn a_group_nothing_could_be_measured_in_has_no_sharpest() {
        let group = Group::new(
            Kind::Series,
            vec![frame("a.jpg", 0, 1), frame("b.jpg", 1, 1)],
        );

        assert!(group.sharpest().is_none());
    }

    #[test]
    fn a_run_of_close_similar_frames_is_one_group() {
        let entries = vec![
            frame("a.jpg", 0, 1),
            frame("b.jpg", 1, 1),
            frame("c.jpg", 2, 1),
        ];

        let groups = detect(&entries, &Settings::default());

        assert_eq!(groups.len(), 1);
        assert_eq!(names(&groups[0]), vec!["a.jpg", "b.jpg", "c.jpg"]);
    }

    #[test]
    fn frames_arrive_in_the_order_they_were_taken_whatever_order_they_were_read() {
        let entries = vec![
            frame("third.jpg", 2, 1),
            frame("first.jpg", 0, 1),
            frame("second.jpg", 1, 1),
        ];

        let groups = detect(&entries, &Settings::default());

        assert_eq!(
            names(&groups[0]),
            vec!["first.jpg", "second.jpg", "third.jpg"]
        );
    }

    #[test]
    fn a_single_frame_is_not_a_group() {
        let entries = vec![frame("lonely.jpg", 0, 1)];

        assert!(detect(&entries, &Settings::default()).is_empty());
    }

    #[test]
    fn a_frame_with_no_capture_time_cannot_be_placed() {
        let entries = vec![
            super::super::test_support::entry("undated.jpg", 0, &[]),
            frame("a.jpg", 0, 1),
            frame("b.jpg", 1, 1),
        ];

        let groups = detect(&entries, &Settings::default());

        assert_eq!(groups.len(), 1);
        assert_eq!(names(&groups[0]), vec!["a.jpg", "b.jpg"]);
    }

    #[test]
    fn what_no_group_claimed_is_reported() {
        let entries = vec![
            frame("a.jpg", 0, 1),
            frame("b.jpg", 1, 1),
            frame("alone.jpg", 600, 9),
        ];

        let groups = detect(&entries, &Settings::default());
        let left = ungrouped(&entries, &groups);

        assert_eq!(left.len(), 1);
        assert_eq!(left[0].name(), "alone.jpg");
    }

    #[test]
    fn a_group_says_how_long_it_took() {
        let group = Group::new(
            Kind::Series,
            vec![frame("a.jpg", 0, 1), frame("b.jpg", 90, 1)],
        );

        assert_eq!(group.span(), Some(90.0));
        assert_eq!(group.describe(), "2 frames over 1 min 30 s");
    }

    #[test]
    fn a_group_taken_within_a_second_says_so() {
        let group = Group::new(
            Kind::Series,
            vec![frame("a.jpg", 0, 1), frame("b.jpg", 0, 1)],
        );

        assert_eq!(group.describe(), "2 frames, all at once");
    }

    #[test]
    fn retyping_a_group_is_remembered() {
        let mut group = Group::new(Kind::Series, vec![frame("a.jpg", 0, 1)]);
        assert!(!group.was_retyped());

        group.kind = Kind::Hdr;
        assert!(group.was_retyped());
        assert_eq!(group.detected, Kind::Series);
    }

    #[test]
    fn a_span_reads_as_a_person_would_say_it() {
        assert_eq!(duration(4.0), "4 s");
        assert_eq!(duration(90.0), "1 min 30 s");
        assert_eq!(duration(7_200.0), "2 h 0 min");
    }

    #[test]
    fn asking_for_larger_groups_drops_the_small_ones() {
        let entries = vec![
            frame("a.jpg", 0, 1),
            frame("b.jpg", 1, 1),
            frame("c.jpg", 600, 2),
            frame("d.jpg", 601, 2),
            frame("e.jpg", 602, 2),
        ];

        let settings = Settings {
            min_frames: 3,
            ..Default::default()
        };

        let groups = detect(&entries, &settings);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }
}

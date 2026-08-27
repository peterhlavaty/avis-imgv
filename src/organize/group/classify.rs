//! Deciding what a run of frames was taken for.
//!
//! Every kind of group is a run of similar frames close in time; what tells
//! them apart is what the photographer changed between them. A bracket changes
//! the exposure and nothing else. A stack changes the focus distance and
//! nothing else. A timelapse changes nothing at all, but arrives on a timer. A
//! burst is what is left: the same settings, as fast as the camera would go.
//!
//! Everything here is a guess from what the files say, and every guess can be
//! overruled in the interface. It is worth being right most of the time and
//! never worth being confident.

use super::super::Entry;
use super::split;

/// What a group of frames was taken for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// The same view at different exposures, for merging.
    Hdr,
    /// The same view at different focus distances, for merging.
    FocusStack,
    /// A camera on a timer.
    Timelapse,
    /// Frames of the same thing, to choose between.
    Series,
}

impl Kind {
    /// Every kind, in the order the dropdown lists them.
    pub const ALL: &'static [Kind] = &[Kind::Hdr, Kind::FocusStack, Kind::Timelapse, Kind::Series];

    pub fn label(self) -> &'static str {
        match self {
            Kind::Hdr => "HDR bracket",
            Kind::FocusStack => "Focus stack",
            Kind::Timelapse => "Timelapse",
            Kind::Series => "Series",
        }
    }

    /// The stem of the folder a group of this kind is tidied into, which the
    /// number is appended to.
    pub fn folder(self) -> &'static str {
        match self {
            Kind::Hdr => "hdr",
            Kind::FocusStack => "stack",
            Kind::Timelapse => "timelapse",
            Kind::Series => "series",
        }
    }
}

/// Frames below this cannot be a bracket or a stack: two frames at different
/// exposures are a change of mind, not a sequence to merge.
const MIN_MERGED: usize = 3;

/// Frames below this cannot be a timelapse, however regular they are.
///
/// A handful of evenly spaced frames is a coincidence; a hundred is a camera
/// on a timer.
const MIN_TIMELAPSE: usize = 8;

/// The longest a bracket takes. Beyond it the camera was not bracketing, it
/// was being adjusted between frames.
const MAX_BRACKET_SPAN: f64 = 30.0;

/// What a run of frames looks like it was taken for.
pub fn kind(members: &[Entry]) -> Kind {
    let gaps = split::gaps(members);

    if is_bracket(members) {
        return Kind::Hdr;
    }

    if is_stack(members) {
        return Kind::FocusStack;
    }

    if members.len() >= MIN_TIMELAPSE && split::is_regular(&gaps) {
        return Kind::Timelapse;
    }

    Kind::Series
}

/// The exposure changes and nothing else does, over a handful of frames taken
/// in quick succession.
fn is_bracket(members: &[Entry]) -> bool {
    if members.len() < MIN_MERGED || span(members) > MAX_BRACKET_SPAN {
        return false;
    }

    let exposure_moves =
        varies(members, "Exposure Compensation") || varies(members, "Exposure Time");

    // The aperture staying put is what separates a bracket from a
    // photographer working out what they want: bracketing changes the time or
    // the compensation and leaves the depth of field alone.
    exposure_moves && !varies(members, "F Number") && !varies(members, "Subject Distance")
}

/// The focus distance changes and the exposure does not.
fn is_stack(members: &[Entry]) -> bool {
    if members.len() < MIN_MERGED {
        return false;
    }

    varies(members, "Subject Distance")
        && !varies(members, "Exposure Time")
        && !varies(members, "F Number")
}

/// Whether a tag has more than one value across the run.
///
/// A tag no file carries does not vary, which is the answer that keeps a
/// camera writing sparse metadata from being read as a bracket.
fn varies(members: &[Entry], tag: &str) -> bool {
    let mut seen: Option<&str> = None;

    for value in members.iter().filter_map(|entry| entry.tag(tag)) {
        match seen {
            None => seen = Some(value),
            Some(first) if first != value => return true,
            Some(_) => {}
        }
    }

    false
}

/// How long the run took, in seconds.
fn span(members: &[Entry]) -> f64 {
    split::gaps(members).iter().sum()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{frame, tagged};
    use super::*;

    /// Three frames a second apart, with the given tags on each.
    fn run(tags: &[&[(&str, &str)]]) -> Vec<Entry> {
        tags.iter()
            .enumerate()
            .map(|(index, pairs)| tagged(frame(&format!("{index}.jpg"), index as i64, 1), pairs))
            .collect()
    }

    #[test]
    fn changing_the_exposure_and_nothing_else_is_a_bracket() {
        let members = run(&[
            &[("Exposure Compensation", "-2"), ("F Number", "8")],
            &[("Exposure Compensation", "0"), ("F Number", "8")],
            &[("Exposure Compensation", "+2"), ("F Number", "8")],
        ]);

        assert_eq!(kind(&members), Kind::Hdr);
    }

    #[test]
    fn a_bracket_by_shutter_speed_counts_too() {
        // Cameras in manual bracket the time rather than the compensation.
        let members = run(&[
            &[("Exposure Time", "1/500"), ("F Number", "8")],
            &[("Exposure Time", "1/125"), ("F Number", "8")],
            &[("Exposure Time", "1/30"), ("F Number", "8")],
        ]);

        assert_eq!(kind(&members), Kind::Hdr);
    }

    #[test]
    fn two_frames_are_never_a_bracket() {
        let members = run(&[
            &[("Exposure Time", "1/500"), ("F Number", "8")],
            &[("Exposure Time", "1/125"), ("F Number", "8")],
        ]);

        assert_eq!(kind(&members), Kind::Series);
    }

    #[test]
    fn changing_the_aperture_too_is_someone_working_it_out() {
        let members = run(&[
            &[("Exposure Time", "1/500"), ("F Number", "2.8")],
            &[("Exposure Time", "1/125"), ("F Number", "5.6")],
            &[("Exposure Time", "1/30"), ("F Number", "11")],
        ]);

        assert_eq!(kind(&members), Kind::Series);
    }

    #[test]
    fn changing_the_focus_and_nothing_else_is_a_stack() {
        let members = run(&[
            &[
                ("Subject Distance", "0.31"),
                ("Exposure Time", "1/60"),
                ("F Number", "8"),
            ],
            &[
                ("Subject Distance", "0.32"),
                ("Exposure Time", "1/60"),
                ("F Number", "8"),
            ],
            &[
                ("Subject Distance", "0.33"),
                ("Exposure Time", "1/60"),
                ("F Number", "8"),
            ],
        ]);

        assert_eq!(kind(&members), Kind::FocusStack);
    }

    #[test]
    fn a_bracket_that_also_refocused_is_read_as_a_bracket_of_nothing() {
        // Both changed, so neither reading holds and it stays a series.
        let members = run(&[
            &[("Subject Distance", "1"), ("Exposure Time", "1/60")],
            &[("Subject Distance", "2"), ("Exposure Time", "1/30")],
            &[("Subject Distance", "3"), ("Exposure Time", "1/15")],
        ]);

        assert_eq!(kind(&members), Kind::Series);
    }

    #[test]
    fn a_long_run_at_a_steady_interval_is_a_timelapse() {
        let members: Vec<Entry> = (0..20)
            .map(|index| frame(&format!("{index}.jpg"), index as i64 * 10, 1))
            .collect();

        assert_eq!(kind(&members), Kind::Timelapse);
    }

    #[test]
    fn a_short_run_at_a_steady_interval_is_not() {
        let members: Vec<Entry> = (0..4)
            .map(|index| frame(&format!("{index}.jpg"), index as i64 * 10, 1))
            .collect();

        assert_eq!(kind(&members), Kind::Series);
    }

    #[test]
    fn a_long_run_at_a_ragged_interval_is_not_either() {
        let times = [0, 1, 9, 11, 30, 31, 32, 50, 51];
        let members: Vec<Entry> = times
            .iter()
            .enumerate()
            .map(|(index, at)| frame(&format!("{index}.jpg"), *at, 1))
            .collect();

        assert_eq!(kind(&members), Kind::Series);
    }

    #[test]
    fn frames_with_nothing_to_tell_them_apart_are_a_series() {
        let members = run(&[
            &[("Exposure Time", "1/500"), ("F Number", "4")],
            &[("Exposure Time", "1/500"), ("F Number", "4")],
            &[("Exposure Time", "1/500"), ("F Number", "4")],
        ]);

        assert_eq!(kind(&members), Kind::Series);
    }

    #[test]
    fn a_camera_that_writes_no_exposure_at_all_is_a_series() {
        let members = run(&[&[], &[], &[]]);

        assert_eq!(kind(&members), Kind::Series);
    }

    #[test]
    fn a_bracket_taken_over_a_minute_is_not_a_bracket() {
        let members: Vec<Entry> = ["-2", "0", "+2"]
            .iter()
            .enumerate()
            .map(|(index, value)| {
                tagged(
                    frame(&format!("{index}.jpg"), index as i64 * 40, 1),
                    &[("Exposure Compensation", value), ("F Number", "8")],
                )
            })
            .collect();

        assert_eq!(kind(&members), Kind::Series);
    }

    #[test]
    fn every_kind_has_a_name_and_a_folder() {
        for kind in Kind::ALL {
            assert!(!kind.label().is_empty());
            assert!(!kind.folder().is_empty());
        }

        assert_eq!(Kind::Hdr.folder(), "hdr");
        assert_eq!(Kind::ALL.len(), 4);
    }
}

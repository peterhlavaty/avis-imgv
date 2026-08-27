//! Cutting a folder into runs of frames that belong together.
//!
//! Two frames continue the same run when they were taken close enough
//! together and show the same thing. Both conditions matter: the clock alone
//! joins two unrelated pictures taken a second apart, and the picture alone
//! joins two visits to the same view a week apart.

use super::super::Entry;
use super::Settings;

/// Splits `entries`, already in the order they were taken, into runs.
pub fn runs(entries: &[Entry], settings: &Settings) -> Vec<Vec<Entry>> {
    let mut runs: Vec<Vec<Entry>> = Vec::new();
    let mut current: Vec<Entry> = Vec::new();

    for entry in entries {
        let continues = current
            .last()
            .is_some_and(|previous| follows(previous, entry, settings));

        if !continues && !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }

        current.push(entry.clone());
    }

    if !current.is_empty() {
        runs.push(current);
    }

    runs
}

/// Whether `entry` continues the run that `previous` is in.
fn follows(previous: &Entry, entry: &Entry, settings: &Settings) -> bool {
    let Some(gap) = gap(previous, entry) else {
        return false;
    };

    if gap > settings.max_gap {
        return false;
    }

    resembles(previous, entry, settings.tolerance)
}

/// Seconds between two frames, or `None` when either is undated.
pub fn gap(previous: &Entry, entry: &Entry) -> Option<f64> {
    let before = previous.captured()?.to_seconds();
    let after = entry.captured()?.to_seconds();

    Some((after - before).abs() as f64)
}

/// Whether two frames show the same thing.
///
/// A file with no thumbnail has no fingerprint, and the answer then rests on
/// the clock alone: a camera that embeds no preview should not stop its owner
/// from grouping a bracket.
fn resembles(previous: &Entry, entry: &Entry, tolerance: u32) -> bool {
    match (previous.fingerprint, entry.fingerprint) {
        (Some(before), Some(after)) => before.resembles(after, tolerance),
        _ => true,
    }
}

/// The seconds between each pair of frames in a run.
pub fn gaps(members: &[Entry]) -> Vec<f64> {
    members
        .windows(2)
        .filter_map(|pair| gap(&pair[0], &pair[1]))
        .collect()
}

/// Whether the frames arrive at a steady interval.
///
/// A timelapse is a camera on a timer, so its gaps are all but identical; a
/// burst is a finger on a button and a stack is a hand on a focus ring, and
/// neither is regular. The allowance is generous because a camera writing to a
/// card can miss its own interval by a second.
pub fn is_regular(gaps: &[f64]) -> bool {
    if gaps.len() < 2 {
        return false;
    }

    let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
    if mean <= 0.0 {
        return false;
    }

    let variance = gaps.iter().map(|gap| (gap - mean).powi(2)).sum::<f64>() / gaps.len() as f64;

    // A second of slack whatever the interval, plus a share of it for the long
    // ones, where a second either way means nothing.
    variance.sqrt() <= (mean * 0.15).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::frame;
    use super::*;

    fn names(runs: &[Vec<Entry>]) -> Vec<Vec<&str>> {
        runs.iter()
            .map(|run| run.iter().map(Entry::name).collect())
            .collect()
    }

    #[test]
    fn a_long_gap_ends_a_run() {
        let entries = vec![
            frame("a.jpg", 0, 1),
            frame("b.jpg", 1, 1),
            frame("c.jpg", 500, 1),
        ];

        assert_eq!(
            names(&runs(&entries, &Settings::default())),
            vec![vec!["a.jpg", "b.jpg"], vec!["c.jpg"]]
        );
    }

    #[test]
    fn a_different_scene_ends_a_run_even_a_second_later() {
        // Two unrelated pictures taken back to back are two pictures.
        let entries = vec![
            frame("a.jpg", 0, 0x0000_0000_0000_0000),
            frame("b.jpg", 1, 0xFFFF_FFFF_FFFF_FFFF),
        ];

        assert_eq!(runs(&entries, &Settings::default()).len(), 2);
    }

    #[test]
    fn the_same_scene_a_week_later_is_not_the_same_run() {
        let entries = vec![frame("a.jpg", 0, 1), frame("b.jpg", 604_800, 1)];

        assert_eq!(runs(&entries, &Settings::default()).len(), 2);
    }

    #[test]
    fn a_file_with_no_thumbnail_is_judged_by_the_clock_alone() {
        let mut second = frame("b.jpg", 1, 1);
        second.fingerprint = None;

        let entries = vec![frame("a.jpg", 0, 1), second];

        assert_eq!(runs(&entries, &Settings::default()).len(), 1);
    }

    #[test]
    fn an_empty_folder_has_no_runs() {
        assert!(runs(&[], &Settings::default()).is_empty());
    }

    #[test]
    fn a_wider_gap_joins_what_a_narrow_one_separates() {
        let entries = vec![frame("a.jpg", 0, 1), frame("b.jpg", 120, 1)];

        assert_eq!(runs(&entries, &Settings::default()).len(), 2);
        assert_eq!(
            runs(
                &entries,
                &Settings {
                    max_gap: 300.0,
                    ..Default::default()
                }
            )
            .len(),
            1
        );
    }

    #[test]
    fn a_camera_on_a_timer_is_regular() {
        assert!(is_regular(&[10.0, 10.0, 10.0, 10.0]));
        assert!(
            is_regular(&[10.0, 11.0, 10.0, 9.0]),
            "a card write is slack"
        );
    }

    #[test]
    fn a_finger_on_a_button_is_not() {
        assert!(!is_regular(&[1.0, 9.0, 2.0, 14.0]));
    }

    #[test]
    fn one_gap_is_not_enough_to_call_an_interval() {
        assert!(!is_regular(&[10.0]));
        assert!(!is_regular(&[]));
    }

    #[test]
    fn frames_taken_in_the_same_second_are_not_an_interval() {
        // Everything is zero, so there is no interval to be regular about.
        assert!(!is_regular(&[0.0, 0.0, 0.0]));
    }

    #[test]
    fn the_gaps_of_a_run_are_the_seconds_between_its_frames() {
        let entries = vec![
            frame("a.jpg", 0, 1),
            frame("b.jpg", 5, 1),
            frame("c.jpg", 15, 1),
        ];

        assert_eq!(gaps(&entries), vec![5.0, 10.0]);
    }
}

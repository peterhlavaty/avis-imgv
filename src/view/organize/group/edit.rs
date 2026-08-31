//! What the buttons in the group panel actually do to the list.
//!
//! Kept apart from the drawing because it is the part with rules: a group of
//! one is not a group, a frame put back belongs where it was taken rather than
//! at the end, and the loose pile stays in the order the shutter fired.

use crate::organize::group::{self, Group};

use super::super::OrganizeView;

/// What the user asked to do to the list of groups.
pub(super) enum Change {
    /// Stop treating this group as one, and let its frames go loose.
    Dissolve(usize),
    /// Take one frame out of a group.
    Remove { group: usize, member: usize },
    /// Put a loose frame into a group.
    Add { group: usize, loose: usize },
}

pub(super) fn apply_change(view: &mut OrganizeView, change: Change) {
    match change {
        Change::Dissolve(index) => {
            let group = view.groups.remove(index);
            view.loose.extend(group.members);
        }
        Change::Remove { group, member } => {
            if member < view.groups[group].members.len() {
                let entry = view.groups[group].members.remove(member);
                view.loose.push(entry);
            }

            // A group of one is not a group.
            if view.groups[group].len() < 2 {
                let emptied = view.groups.remove(group);
                view.loose.extend(emptied.members);
            }
        }
        Change::Add { group, loose } => {
            if loose < view.loose.len() {
                let entry = view.loose.remove(loose);
                view.groups[group].members.push(entry);
                sort_members(&mut view.groups[group]);
            }
        }
    }

    sort_loose(&mut view.loose);
}

/// Puts a group's frames back in the order they were taken, after one has been
/// dropped into it.
fn sort_members(group: &mut Group) {
    group.members.sort_by_key(|entry| {
        (
            entry
                .captured()
                .map(|at| at.to_seconds())
                .unwrap_or(i64::MAX),
            entry.name().to_string(),
        )
    });
}

fn sort_loose(loose: &mut [crate::organize::Entry]) {
    loose.sort_by_key(|entry| {
        (
            entry
                .captured()
                .map(|at| at.to_seconds())
                .unwrap_or(i64::MAX),
            entry.name().to_string(),
        )
    });
}

/// Reads the folder into groups, and everything that fell outside them.
pub(in crate::view::organize) fn regroup(
    entries: &[crate::organize::Entry],
    settings: &group::Settings,
) -> (Vec<Group>, Vec<crate::organize::Entry>) {
    let groups = group::detect(entries, settings);
    let mut loose = group::ungrouped(entries, &groups);

    sort_loose(&mut loose);

    (groups, loose)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organize::group::test_support::frame;
    use crate::organize::group::Kind;
    use crate::organize::Entry;

    fn group_of(names: &[(&str, i64)]) -> Group {
        Group::new(
            Kind::Series,
            names.iter().map(|(name, at)| frame(name, *at, 1)).collect(),
        )
    }

    fn view_with(groups: Vec<Group>, loose: Vec<Entry>) -> OrganizeView {
        OrganizeView {
            groups,
            loose,
            ..OrganizeView::default()
        }
    }

    fn names(entries: &[Entry]) -> Vec<&str> {
        entries.iter().map(Entry::name).collect()
    }

    #[test]
    fn dissolving_a_group_puts_its_frames_back_in_the_loose_pile() {
        let mut view = view_with(vec![group_of(&[("a.jpg", 0), ("b.jpg", 1)])], Vec::new());

        apply_change(&mut view, Change::Dissolve(0));

        assert!(view.groups.is_empty());
        assert_eq!(names(&view.loose), vec!["a.jpg", "b.jpg"]);
    }

    #[test]
    fn taking_a_frame_out_leaves_the_rest_grouped() {
        let mut view = view_with(
            vec![group_of(&[("a.jpg", 0), ("b.jpg", 1), ("c.jpg", 2)])],
            Vec::new(),
        );

        apply_change(
            &mut view,
            Change::Remove {
                group: 0,
                member: 1,
            },
        );

        assert_eq!(names(&view.groups[0].members), vec!["a.jpg", "c.jpg"]);
        assert_eq!(names(&view.loose), vec!["b.jpg"]);
    }

    #[test]
    fn taking_the_second_to_last_frame_out_dissolves_the_group() {
        let mut view = view_with(vec![group_of(&[("a.jpg", 0), ("b.jpg", 1)])], Vec::new());

        apply_change(
            &mut view,
            Change::Remove {
                group: 0,
                member: 0,
            },
        );

        assert!(view.groups.is_empty(), "one frame is not a group");
        assert_eq!(names(&view.loose), vec!["a.jpg", "b.jpg"]);
    }

    #[test]
    fn a_frame_put_into_a_group_lands_in_the_order_it_was_taken() {
        let mut view = view_with(
            vec![group_of(&[("a.jpg", 0), ("c.jpg", 2)])],
            vec![frame("b.jpg", 1, 1)],
        );

        apply_change(&mut view, Change::Add { group: 0, loose: 0 });

        assert_eq!(
            names(&view.groups[0].members),
            vec!["a.jpg", "b.jpg", "c.jpg"]
        );
        assert!(view.loose.is_empty());
    }

    #[test]
    fn the_loose_pile_stays_in_the_order_the_frames_were_taken() {
        let mut view = view_with(
            vec![group_of(&[("a.jpg", 10), ("b.jpg", 11)])],
            vec![frame("late.jpg", 99, 2)],
        );

        apply_change(&mut view, Change::Dissolve(0));

        assert_eq!(names(&view.loose), vec!["a.jpg", "b.jpg", "late.jpg"]);
    }

    #[test]
    fn regrouping_returns_both_the_groups_and_what_fell_outside_them() {
        let entries = vec![
            frame("a.jpg", 0, 1),
            frame("b.jpg", 1, 1),
            frame("alone.jpg", 9999, 2),
        ];

        let (groups, loose) = regroup(&entries, &group::Settings::default());

        assert_eq!(groups.len(), 1);
        assert_eq!(names(&loose), vec!["alone.jpg"]);
    }
}

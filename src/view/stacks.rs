//! Stacking a burst into one cell, without moving a file.
//!
//! A folder that has been shot properly is mostly repetition: five frames of
//! one expression, three exposures of one view, a hundred from a camera on a
//! timer. A contact sheet that shows all of them shows the same photograph
//! five times over, and the frame worth keeping is somewhere in the middle of
//! it.
//!
//! Every other program answers this by stacking: the run becomes one cell with
//! a number on it, and opening the stack shows what is inside. Lightroom keeps
//! the stack in its catalogue, Bridge in a hidden file beside the photographs.
//! This keeps it nowhere. The detector already reads a folder into groups for
//! the organiser, and a group is a stack — so the stacks are worked out from
//! what the files themselves say, every time, and turning them off leaves
//! nothing behind to clean up.
//!
//! The mechanism is [`Visible`], the same list of store positions a filter
//! narrows: a closed stack keeps its standing frame on the list and leaves the
//! rest off. So stacking composes with filtering and ordering for nothing, and
//! no photograph is decoded twice for it.

use std::collections::HashMap;
use std::path::Path;

use crate::organize::group::{Group, Kind};
use crate::view::visible::Visible;

/// One detected run of frames, as the browsing views hold it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
    pub kind: Kind,
    /// The store positions of its frames, in the order the sheet shows them.
    pub members: Vec<usize>,
    /// Which member stands for the whole run while it is closed, as a position
    /// within `members`.
    pub standing: usize,
    pub collapsed: bool,
}

impl Stack {
    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// The store position of the frame standing for the run.
    pub fn stands(&self) -> usize {
        self.members[self.standing.min(self.members.len() - 1)]
    }

    /// Where a store position sits in the run, counting from one.
    fn frame_of(&self, index: usize) -> Option<usize> {
        self.members
            .iter()
            .position(|member| *member == index)
            .map(|at| at + 1)
    }
}

/// Where a photograph stands: which frame of which run.
///
/// What the status bar says, and the reason it says it — somebody a third of
/// the way through a burst of seventeen wants to know that is where they are,
/// and a file name does not tell them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    pub kind: Kind,
    /// Which run it is, counting from one.
    pub stack: usize,
    pub stacks: usize,
    /// Which frame of that run, counting from one.
    pub frame: usize,
    pub frames: usize,
    pub collapsed: bool,
}

impl Place {
    /// `Focus stack 3 · frame 4 of 17 · stack 6 of 41`, for the bar under the
    /// photograph.
    ///
    /// `Kind::label` and not `Kind::folder`, whose documented job is to name a
    /// *folder*: a focus stack read "stack 3 · frame 4 of 17 · stack 3 of 41",
    /// which uses one word for two things and prints the same number twice.
    pub fn describe(&self) -> String {
        format!(
            "{} {} · frame {} of {} · stack {} of {}",
            self.kind.label(),
            self.stack,
            self.frame,
            self.frames,
            self.stack,
            self.stacks
        )
    }
}

/// Every stack found in the open folder.
#[derive(Debug, Clone, Default)]
pub struct Stacks {
    stacks: Vec<Stack>,
    /// Which stack each store position belongs to, if any.
    of: HashMap<usize, usize>,
}

impl Stacks {
    /// Reads detected groups into stacks over the store's positions.
    ///
    /// `position_of` maps a file to where the store holds it, because the
    /// detector works in capture order over whatever it could read, and the
    /// views work in store positions over everything that was opened.
    pub fn of_groups(
        groups: &[Group],
        position_of: impl Fn(&Path) -> Option<usize>,
        collapsed: bool,
    ) -> Stacks {
        let mut stacks: Vec<Stack> = Vec::new();

        for group in groups {
            let members: Vec<usize> = group
                .members
                .iter()
                .filter_map(|entry| position_of(&entry.path))
                .collect();

            // A run the store holds barely any of is not a stack here,
            // whatever the detector made of it.
            if members.len() < 2 {
                continue;
            }

            // The sharpest frame stands for the run where one could be
            // measured: choosing between five frames of one thing is mostly
            // the question of which is in focus, so the sheet may as well
            // answer it before being asked.
            let sharpest = group
                .sharpest()
                .and_then(|at| group.members.get(at))
                .and_then(|entry| position_of(&entry.path));

            let mut members = members;
            members.sort_unstable();

            let standing = sharpest
                .and_then(|index| members.iter().position(|member| *member == index))
                .unwrap_or(0);

            stacks.push(Stack {
                kind: group.kind,
                members,
                standing,
                collapsed,
            });
        }

        // In the order the store holds them, so "the next stack" means the
        // next one down the sheet rather than the next one the detector
        // happened to finish.
        stacks.sort_by_key(|stack| stack.members[0]);

        let mut of = HashMap::new();
        for (at, stack) in stacks.iter().enumerate() {
            for member in &stack.members {
                of.insert(*member, at);
            }
        }

        Stacks { stacks, of }
    }

    pub fn is_empty(&self) -> bool {
        self.stacks.is_empty()
    }

    /// How many runs were found.
    pub fn len(&self) -> usize {
        self.stacks.len()
    }

    /// How many photographs are in a stack at all.
    pub fn stacked(&self) -> usize {
        self.of.len()
    }

    pub fn stack_of(&self, index: usize) -> Option<&Stack> {
        self.of.get(&index).and_then(|at| self.stacks.get(*at))
    }

    /// Leaves out the frames of closed stacks that are not standing for them.
    ///
    /// A stack whose standing frame the filter has hidden falls back to the
    /// first of its frames that survived, rather than disappearing: "hide the
    /// rejected" should take frames out of a burst, not take the burst out of
    /// the folder.
    pub fn fold(&self, visible: &Visible, total: usize) -> Visible {
        if self.stacks.is_empty() {
            return visible.clone();
        }

        let showing: Vec<usize> = visible.iter().collect();
        let mut standing_for: HashMap<usize, usize> = HashMap::new();

        for index in &showing {
            let Some(at) = self.of.get(index) else {
                continue;
            };

            let stack = &self.stacks[*at];
            if !stack.collapsed {
                continue;
            }

            let stands = stack.stands();
            let shown = standing_for.entry(*at).or_insert(*index);
            if *index == stands {
                *shown = *index;
            }
        }

        let kept: Vec<usize> = showing
            .into_iter()
            .filter(|index| match self.of.get(index) {
                Some(at) => standing_for.get(at).is_none_or(|shown| shown == index),
                None => true,
            })
            .collect();

        Visible::of(kept, total)
    }

    /// Opens or closes the stack a photograph is in. Returns whether there was
    /// one to open.
    pub fn toggle(&mut self, index: usize) -> bool {
        let Some(at) = self.of.get(&index).copied() else {
            return false;
        };

        self.stacks[at].collapsed = !self.stacks[at].collapsed;
        true
    }

    /// Closes every stack, or opens every one.
    pub fn set_all(&mut self, collapsed: bool) {
        for stack in &mut self.stacks {
            stack.collapsed = collapsed;
        }
    }

    /// Whether every stack is closed.
    pub fn all_collapsed(&self) -> bool {
        self.stacks.iter().all(|stack| stack.collapsed)
    }

    /// Changes which frame stands for the stack a photograph is in, and
    /// returns the store position of the frame that now does.
    ///
    /// The point of a closed stack is that one frame is worth looking at;
    /// which one that is, is a judgement the detector cannot make, so it takes
    /// two keys to walk the run without opening it.
    pub fn step_standing(&mut self, index: usize, forward: bool) -> Option<usize> {
        let at = self.of.get(&index).copied()?;
        let stack = &mut self.stacks[at];
        let last = stack.len() - 1;

        stack.standing = match (forward, stack.standing.min(last)) {
            (true, current) if current >= last => 0,
            (true, current) => current + 1,
            (false, 0) => last,
            (false, current) => current - 1,
        };

        Some(stack.stands())
    }

    /// The frame to go to for the stack after — or before — the one a
    /// photograph is in, so a burst can be stepped over rather than through.
    ///
    /// From a photograph in no stack, the nearest stack in that direction.
    pub fn step_stack(&self, index: usize, forward: bool) -> Option<usize> {
        if self.stacks.is_empty() {
            return None;
        }

        let last = self.stacks.len() - 1;
        let at = match self.of.get(&index) {
            Some(at) => match (forward, *at) {
                (true, current) if current >= last => 0,
                (true, current) => current + 1,
                (false, 0) => last,
                (false, current) => current - 1,
            },
            None => self.nearest(index, forward),
        };

        let stack = &self.stacks[at];
        Some(if stack.collapsed {
            stack.stands()
        } else {
            stack.members[0]
        })
    }

    /// The stack nearest `index` in the given direction, wrapping at the ends.
    fn nearest(&self, index: usize, forward: bool) -> usize {
        let starts = |at: usize| self.stacks[at].members[0];
        let last = self.stacks.len() - 1;

        if forward {
            (0..self.stacks.len())
                .find(|at| starts(*at) > index)
                .unwrap_or(0)
        } else {
            (0..self.stacks.len())
                .rev()
                .find(|at| starts(*at) < index)
                .unwrap_or(last)
        }
    }

    /// The first frame of every open stack, which is how the driver remembers
    /// them across a redetection.
    ///
    /// By the frame rather than by the stack's number: the numbers change the
    /// moment the tolerance does, and the photograph a run starts on is the
    /// one thing about it a person would recognise.
    pub fn opened(&self) -> std::collections::HashSet<usize> {
        self.stacks
            .iter()
            .filter(|stack| !stack.collapsed)
            .map(|stack| stack.members[0])
            .collect()
    }

    /// Opens the stacks that start on one of `firsts`.
    pub fn open(&mut self, firsts: &std::collections::HashSet<usize>) {
        for stack in &mut self.stacks {
            if firsts.contains(&stack.members[0]) {
                stack.collapsed = false;
            }
        }
    }

    /// Where a photograph stands, for the status bar.
    pub fn place_of(&self, index: usize) -> Option<Place> {
        let at = *self.of.get(&index)?;
        let stack = &self.stacks[at];

        Some(Place {
            kind: stack.kind,
            stack: at + 1,
            stacks: self.stacks.len(),
            frame: stack.frame_of(index)?,
            frames: stack.len(),
            collapsed: stack.collapsed,
        })
    }
}

/// The glyph a stack of each kind wears in the corner of its cell.
///
/// Shapes rather than colours: the cell already says the rating and the flag
/// in colour, and a burst and a bracket are different questions rather than
/// different amounts of the same one.
///
/// Every one of them has to be in a font the proportional chain actually
/// loads — the bundled Atkinson, then Ubuntu-Light, NotoEmoji and
/// emoji-icon-font. `◐` and `❏` were not: `◐` is in Hack, which is the
/// *monospace* family, and `❏` is in none of them, so both drew an empty box
/// wherever they appeared. The legend is what made that visible.
pub fn glyph(kind: Kind) -> &'static str {
    match kind {
        Kind::Hdr => "◑",
        Kind::FocusStack => "◎",
        Kind::Timelapse => "⏱",
        Kind::Series => "▣",
    }
}

/// The store's paths as a lookup, for turning detected groups into stacks.
pub fn positions(paths: &[std::path::PathBuf]) -> HashMap<&Path, usize> {
    paths
        .iter()
        .enumerate()
        .map(|(index, path)| (path.as_path(), index))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organize::group::test_support::frame;
    use crate::organize::group::Group;
    use std::path::PathBuf;

    /// Two runs of three, with a single frame between them.
    fn folder() -> (Vec<PathBuf>, Vec<Group>) {
        // The same paths the group test helper builds, so a frame can be
        // found in the store's list.
        let paths: Vec<PathBuf> = (0..7)
            .map(|n| PathBuf::from("/photos").join(format!("f{n}.jpg")))
            .collect();

        let burst = Group::new(
            Kind::Series,
            vec![
                frame("f0.jpg", 0, 1),
                frame("f1.jpg", 1, 1),
                frame("f2.jpg", 2, 1),
            ],
        );
        let other = Group::new(
            Kind::Hdr,
            vec![
                frame("f4.jpg", 400, 2),
                frame("f5.jpg", 401, 2),
                frame("f6.jpg", 402, 2),
            ],
        );

        (paths, vec![burst, other])
    }

    fn stacks(collapsed: bool) -> (Vec<PathBuf>, Stacks) {
        let (paths, groups) = folder();
        let owned = paths.clone();
        let stacks = Stacks::of_groups(
            &groups,
            move |path| owned.iter().position(|known| known == path),
            collapsed,
        );

        (paths, stacks)
    }

    #[test]
    fn a_closed_stack_shows_one_frame_and_an_open_one_shows_them_all() {
        let (paths, mut stacks) = stacks(true);
        let everything = Visible::everything(paths.len());

        // Three frames, then the loose one, then three more: five cells.
        let folded = stacks.fold(&everything, paths.len());
        assert_eq!(folded.iter().collect::<Vec<_>>(), vec![0, 3, 4]);

        stacks.set_all(false);
        assert_eq!(
            stacks.fold(&everything, paths.len()).iter().count(),
            paths.len()
        );
    }

    #[test]
    fn opening_one_stack_leaves_the_others_closed() {
        let (paths, mut stacks) = stacks(true);
        let everything = Visible::everything(paths.len());

        assert!(stacks.toggle(1));
        assert_eq!(
            stacks
                .fold(&everything, paths.len())
                .iter()
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn a_photograph_in_no_stack_is_never_hidden() {
        let (paths, stacks) = stacks(true);
        let everything = Visible::everything(paths.len());

        assert!(stacks.stack_of(3).is_none());
        assert!(stacks
            .fold(&everything, paths.len())
            .position_of(3)
            .is_some());
    }

    /// The stack stands on whichever of its frames survived, rather than
    /// vanishing with the one that did not.
    #[test]
    fn a_filter_that_hides_the_standing_frame_does_not_hide_the_stack() {
        let (paths, stacks) = stacks(true);
        let narrowed = Visible::of(vec![1, 2, 3, 4, 5, 6], paths.len());

        let folded = stacks.fold(&narrowed, paths.len());

        assert_eq!(folded.iter().collect::<Vec<_>>(), vec![1, 3, 4]);
    }

    #[test]
    fn the_standing_frame_walks_the_run_and_wraps() {
        let (paths, mut stacks) = stacks(true);
        let everything = Visible::everything(paths.len());

        assert_eq!(stacks.step_standing(0, true), Some(1));
        assert_eq!(
            stacks
                .fold(&everything, paths.len())
                .iter()
                .collect::<Vec<_>>(),
            vec![1, 3, 4]
        );

        assert_eq!(stacks.step_standing(0, true), Some(2));
        assert_eq!(stacks.step_standing(0, true), Some(0));
        assert_eq!(stacks.step_standing(0, false), Some(2));
    }

    #[test]
    fn stepping_stacks_goes_over_a_burst_rather_than_through_it() {
        let (_, stacks) = stacks(true);

        // From inside the first run to the second, and round again.
        assert_eq!(stacks.step_stack(1, true), Some(4));
        assert_eq!(stacks.step_stack(4, true), Some(0));
        assert_eq!(stacks.step_stack(0, false), Some(4));

        // From the loose frame between them, to the one on either side.
        assert_eq!(stacks.step_stack(3, true), Some(4));
        assert_eq!(stacks.step_stack(3, false), Some(0));
    }

    #[test]
    fn a_photograph_says_which_frame_of_which_run_it_is() {
        let (_, stacks) = stacks(true);

        let place = stacks.place_of(5).unwrap();
        assert_eq!(place.frame, 2);
        assert_eq!(place.frames, 3);
        assert_eq!(place.stack, 2);
        assert_eq!(place.stacks, 2);
        assert_eq!(place.kind, Kind::Hdr);

        assert!(stacks.place_of(3).is_none());
        assert_eq!(
            stacks.place_of(0).unwrap().describe(),
            "Series 1 · frame 1 of 3 · stack 1 of 2"
        );
    }

    /// The store may not hold every frame the detector grouped — a filter on
    /// file type, a folder read while files are arriving — and a run reduced
    /// to one frame is not a stack.
    #[test]
    fn a_run_the_store_barely_holds_is_not_a_stack() {
        let (_, groups) = folder();
        let only = PathBuf::from("/photos").join("f1.jpg");

        let stacks = Stacks::of_groups(&groups, |path| (path == only).then_some(0), true);

        assert!(stacks.is_empty());
    }

    #[test]
    fn nothing_detected_leaves_the_folder_exactly_as_it_was() {
        let everything = Visible::everything(7);
        let mut stacks = Stacks::default();

        assert!(stacks.fold(&everything, 7).is_everything());
        assert_eq!(stacks.step_stack(3, true), None);
        assert!(!stacks.toggle(3));
    }
}

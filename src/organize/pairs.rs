//! Raw and JPEG shot together, browsed as one photograph.
//!
//! A camera set to raw+JPEG writes two files of the same frame. A viewer that
//! does not know they belong together shows the shoot twice, makes you rate
//! everything twice, and — worse — lets the two copies disagree: reject the
//! JPEG, keep the raw, and what survives the cull is the opposite of what was
//! decided. Every program a photographer would compare this one to treats the
//! pair as one thing.
//!
//! So one of them is browsed and the other follows it. Which one is browsed is
//! a setting, because the two answers are both reasonable: the JPEG decodes in
//! a tenth of the time, and the raw is the file that will actually be
//! developed. What follows is everything else — a rating, a flag, a colour
//! label, a keyword, a move, a copy, a deletion. The partner is never shown
//! and never separately markable, which is the point.
//!
//! Files are paired by the name the camera gave them: same folder, same stem,
//! one of them raw and one of them not. That is the convention every camera
//! follows and the only one that can be relied on — a timestamp would pair
//! frames from two bodies, and reading both files to compare them would cost a
//! folder's worth of decoding to answer a question the file name already
//! answers.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::formats::{self, Format};

/// Which half of a raw+JPEG pair is the one browsed.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Prefer {
    /// Do not pair them at all; both files are their own photograph.
    Off,
    /// Browse the JPEG, which decodes in a fraction of the time.
    #[default]
    Jpeg,
    /// Browse the raw, which is the file that will be developed.
    Raw,
}

impl Prefer {
    pub const ALL: &'static [Prefer] = &[Prefer::Off, Prefer::Jpeg, Prefer::Raw];

    pub fn label(self) -> &'static str {
        match self {
            Prefer::Off => "Show both",
            Prefer::Jpeg => "Show the JPEG",
            Prefer::Raw => "Show the raw",
        }
    }
}

/// Which files follow which, once a folder has been paired.
#[derive(Debug, Default, Clone)]
pub struct Pairs {
    /// Keyed by the file that is browsed; the value is what goes with it.
    partners: HashMap<PathBuf, Vec<PathBuf>>,
}

impl Pairs {
    /// Splits `paths` into what is browsed and what follows it.
    ///
    /// The order of what is browsed is the order it came in, so a collection
    /// that was sorted stays sorted.
    pub fn gather(paths: &[PathBuf], prefer: Prefer) -> (Vec<PathBuf>, Pairs) {
        if prefer == Prefer::Off {
            return (paths.to_vec(), Pairs::default());
        }

        let mut groups: HashMap<(PathBuf, String), Vec<PathBuf>> = HashMap::new();
        for path in paths {
            groups.entry(key(path)).or_default().push(path.clone());
        }

        let mut pairs = Pairs::default();
        let mut shown = Vec::with_capacity(paths.len());

        for path in paths {
            let Some(group) = groups.get(&key(path)) else {
                continue;
            };

            let Some(chosen) = choose(group, prefer) else {
                // Not a pair: everything in the group is its own photograph.
                shown.push(path.clone());
                continue;
            };

            if chosen != path {
                continue;
            }

            let following: Vec<PathBuf> = group
                .iter()
                .filter(|other| *other != path)
                .cloned()
                .collect();

            shown.push(path.clone());
            pairs.partners.insert(path.clone(), following);
        }

        (shown, pairs)
    }

    /// What follows `path`, which is empty for a photograph shot alone.
    pub fn partners_of(&self, path: &Path) -> &[PathBuf] {
        self.partners.get(path).map_or(&[], Vec::as_slice)
    }

    /// `path` and everything that follows it.
    ///
    /// What every command that touches a file works on, so that none of them
    /// has to know that pairing exists.
    pub fn with_partners(&self, path: &Path) -> Vec<PathBuf> {
        let mut all = vec![path.to_path_buf()];
        all.extend(self.partners_of(path).iter().cloned());

        all
    }

    /// Every file in the collection, browsed or following.
    ///
    /// What the folder jobs want: a bulk rename that renamed only half of
    /// every pair would break the pairing it was meant to preserve.
    pub fn everything(&self, shown: &[PathBuf]) -> Vec<PathBuf> {
        let mut all = Vec::with_capacity(shown.len());
        for path in shown {
            all.extend(self.with_partners(path));
        }

        all
    }

    /// How many photographs have a partner, for the status bar.
    pub fn len(&self) -> usize {
        self.partners.len()
    }

    pub fn is_empty(&self) -> bool {
        self.partners.is_empty()
    }

    /// Attaches a file that has just appeared to the photograph it belongs
    /// to, if there is one.
    ///
    /// Returns whether it was taken as a partner, in which case it does not
    /// join the collection in its own right — the second half of a raw+JPEG
    /// pair landing during a tethered shoot must not appear as a second
    /// photograph of the same frame.
    ///
    /// Which half is *browsed* does not change here. Swapping the picture on
    /// screen for a different file because a partner arrived a moment later
    /// would be a worse surprise than showing the one that arrived first; the
    /// preference decides again the next time the folder is opened.
    pub fn take_in(&mut self, arriving: &Path, shown: &[PathBuf], prefer: Prefer) -> bool {
        if prefer == Prefer::Off {
            return false;
        }

        let wanted = key(arriving);
        let Some(owner) = shown.iter().find(|path| key(path) == wanted) else {
            return false;
        };

        // Only a raw and a not-raw make a pair; two of a kind are two
        // photographs, the same as they would be on a folder read.
        if is_raw(owner) == is_raw(arriving) {
            return false;
        }

        let following = self.partners.entry(owner.clone()).or_default();
        if !following.iter().any(|path| path == arriving) {
            following.push(arriving.to_path_buf());
        }

        true
    }

    /// Forgets a photograph that has left the collection.
    pub fn forget(&mut self, path: &Path) {
        self.partners.remove(path);
    }
}

/// What makes two files the same frame: the folder and the name the camera
/// gave them, without the extension.
///
/// Compared without regard to case, because a card written by one camera and
/// read by one program can easily hold `IMG_1234.CR2` beside `img_1234.jpg`.
fn key(path: &Path) -> (PathBuf, String) {
    let folder = path.parent().unwrap_or(Path::new("")).to_path_buf();
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    (folder, stem)
}

/// The one of `group` to browse, or `None` when the group is not a pair.
///
/// A group is a pair only when it holds a raw *and* something that is not one.
/// Two JPEGs of the same stem — `a.jpg` beside `a.jpeg` — are two photographs
/// as far as anybody can tell from the outside, and hiding one of them would
/// be a way to lose a picture.
fn choose(group: &[PathBuf], prefer: Prefer) -> Option<&PathBuf> {
    let (raw, other): (Vec<&PathBuf>, Vec<&PathBuf>) = group.iter().partition(|path| is_raw(path));

    if raw.is_empty() || other.is_empty() {
        return None;
    }

    let wanted = match prefer {
        Prefer::Raw => raw.first(),
        // Off never reaches here, and browsing the JPEG is what it would mean
        // anyway.
        Prefer::Jpeg | Prefer::Off => other.first(),
    };

    wanted.copied()
}

fn is_raw(path: &Path) -> bool {
    formats::Format::from_path(path) == Some(Format::Raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn a_pair_is_browsed_as_one_photograph() {
        let all = paths(&["/p/IMG_1.CR2", "/p/IMG_1.JPG", "/p/IMG_2.JPG"]);
        let (shown, pairs) = Pairs::gather(&all, Prefer::Jpeg);

        assert_eq!(shown, paths(&["/p/IMG_1.JPG", "/p/IMG_2.JPG"]));
        assert_eq!(
            pairs.partners_of(Path::new("/p/IMG_1.JPG")),
            paths(&["/p/IMG_1.CR2"])
        );
        assert!(pairs.partners_of(Path::new("/p/IMG_2.JPG")).is_empty());
    }

    #[test]
    fn the_setting_decides_which_half_is_browsed() {
        let all = paths(&["/p/IMG_1.CR2", "/p/IMG_1.JPG"]);

        let (shown, pairs) = Pairs::gather(&all, Prefer::Raw);
        assert_eq!(shown, paths(&["/p/IMG_1.CR2"]));
        assert_eq!(
            pairs.partners_of(Path::new("/p/IMG_1.CR2")),
            paths(&["/p/IMG_1.JPG"])
        );
    }

    #[test]
    fn pairing_can_be_turned_off_entirely() {
        let all = paths(&["/p/IMG_1.CR2", "/p/IMG_1.JPG"]);
        let (shown, pairs) = Pairs::gather(&all, Prefer::Off);

        assert_eq!(shown, all);
        assert!(pairs.is_empty());
    }

    /// The order a collection came in is the order it stays in, whichever
    /// half of each pair is browsed.
    #[test]
    fn the_order_survives_pairing() {
        let all = paths(&[
            "/p/A.JPG", "/p/A.NEF", "/p/B.JPG", "/p/C.NEF", "/p/C.JPG", "/p/D.JPG",
        ]);

        let (shown, _) = Pairs::gather(&all, Prefer::Raw);
        assert_eq!(
            shown,
            paths(&["/p/A.NEF", "/p/B.JPG", "/p/C.NEF", "/p/D.JPG"])
        );
    }

    /// The rule that stops a picture disappearing: without a raw in it, a
    /// group is not a pair.
    #[test]
    fn two_files_that_are_both_pictures_are_both_shown() {
        let all = paths(&["/p/a.jpg", "/p/a.jpeg", "/p/a.png"]);
        let (shown, pairs) = Pairs::gather(&all, Prefer::Jpeg);

        assert_eq!(shown, all);
        assert!(pairs.is_empty());
    }

    /// And neither does a group of raws alone.
    #[test]
    fn two_raws_of_the_same_name_are_both_shown() {
        let all = paths(&["/p/a.cr2", "/p/a.nef"]);
        let (shown, _) = Pairs::gather(&all, Prefer::Jpeg);

        assert_eq!(shown, all);
    }

    #[test]
    fn the_same_name_in_two_folders_is_two_photographs() {
        let all = paths(&["/a/IMG_1.JPG", "/b/IMG_1.JPG", "/b/IMG_1.CR2"]);
        let (shown, pairs) = Pairs::gather(&all, Prefer::Jpeg);

        assert_eq!(shown, paths(&["/a/IMG_1.JPG", "/b/IMG_1.JPG"]));
        assert!(pairs.partners_of(Path::new("/a/IMG_1.JPG")).is_empty());
        assert_eq!(pairs.partners_of(Path::new("/b/IMG_1.JPG")).len(), 1);
    }

    /// A card written by one program and read by another can hold either
    /// case, and they are still the same frame.
    #[test]
    fn case_does_not_separate_a_pair() {
        let all = paths(&["/p/IMG_1234.CR2", "/p/img_1234.jpg"]);
        let (shown, pairs) = Pairs::gather(&all, Prefer::Jpeg);

        assert_eq!(shown.len(), 1);
        assert_eq!(pairs.partners_of(&shown[0]).len(), 1);
    }

    /// What every command that touches a file asks for.
    #[test]
    fn a_command_acts_on_the_pair() {
        let all = paths(&["/p/IMG_1.CR2", "/p/IMG_1.JPG", "/p/IMG_2.JPG"]);
        let (shown, pairs) = Pairs::gather(&all, Prefer::Jpeg);

        assert_eq!(
            pairs.with_partners(Path::new("/p/IMG_1.JPG")),
            paths(&["/p/IMG_1.JPG", "/p/IMG_1.CR2"])
        );
        assert_eq!(
            pairs.with_partners(Path::new("/p/IMG_2.JPG")),
            paths(&["/p/IMG_2.JPG"])
        );

        // And the folder jobs see every file there is.
        assert_eq!(pairs.everything(&shown).len(), all.len());
    }

    /// A tethered shoot lands the two halves a moment apart, and the second
    /// must not appear as a second photograph.
    #[test]
    fn the_second_half_of_a_pair_arriving_later_is_taken_as_a_partner() {
        let shown = paths(&["/p/IMG_1.JPG"]);
        let (_, mut pairs) = Pairs::gather(&shown, Prefer::Jpeg);

        assert!(pairs.take_in(Path::new("/p/IMG_1.CR2"), &shown, Prefer::Jpeg));
        assert_eq!(
            pairs.partners_of(Path::new("/p/IMG_1.JPG")),
            paths(&["/p/IMG_1.CR2"])
        );
    }

    #[test]
    fn an_unrelated_arrival_is_a_photograph_of_its_own() {
        let shown = paths(&["/p/IMG_1.JPG"]);
        let (_, mut pairs) = Pairs::gather(&shown, Prefer::Jpeg);

        assert!(!pairs.take_in(Path::new("/p/IMG_2.CR2"), &shown, Prefer::Jpeg));
        assert!(!pairs.take_in(Path::new("/p/IMG_1.PNG"), &shown, Prefer::Jpeg));
        assert!(pairs.is_empty());
    }

    #[test]
    fn nothing_is_paired_when_pairing_is_off() {
        let shown = paths(&["/p/IMG_1.JPG"]);
        let mut pairs = Pairs::default();

        assert!(!pairs.take_in(Path::new("/p/IMG_1.CR2"), &shown, Prefer::Off));
    }

    #[test]
    fn the_same_partner_arriving_twice_is_recorded_once() {
        let shown = paths(&["/p/IMG_1.JPG"]);
        let (_, mut pairs) = Pairs::gather(&shown, Prefer::Jpeg);

        assert!(pairs.take_in(Path::new("/p/IMG_1.CR2"), &shown, Prefer::Jpeg));
        assert!(pairs.take_in(Path::new("/p/IMG_1.CR2"), &shown, Prefer::Jpeg));
        assert_eq!(pairs.partners_of(Path::new("/p/IMG_1.JPG")).len(), 1);
    }

    #[test]
    fn a_photograph_that_has_gone_is_forgotten() {
        let all = paths(&["/p/IMG_1.CR2", "/p/IMG_1.JPG"]);
        let (_, mut pairs) = Pairs::gather(&all, Prefer::Jpeg);

        assert_eq!(pairs.len(), 1);
        pairs.forget(Path::new("/p/IMG_1.JPG"));
        assert!(pairs.is_empty());
    }

    #[test]
    fn an_empty_collection_pairs_to_nothing() {
        let (shown, pairs) = Pairs::gather(&[], Prefer::Jpeg);

        assert!(shown.is_empty());
        assert!(pairs.is_empty());
    }

    #[test]
    fn every_setting_has_a_name() {
        for prefer in Prefer::ALL {
            assert!(!prefer.label().is_empty());
        }

        assert_eq!(Prefer::ALL.len(), 3);
    }
}

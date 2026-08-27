//! Correcting a camera clock after the fact.
//!
//! A camera set to the wrong time zone, or never set at all, stamps a whole
//! trip with the wrong time. Every photograph is wrong by the same amount, so
//! the fix is one offset applied to all of them.
//!
//! Which timestamps move is the user's choice: a file carries the moment the
//! shutter opened and the moment the file was written, and there are reasons
//! to want one without the other.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::formats::Format;
use crate::metadata::dates::{self, DateField};
use crate::metadata::datetime::Timestamp;

use super::Entry;

/// How far to move the clock, and which way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offset {
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
    /// True when the photographs were taken later than the camera thought.
    pub forward: bool,
}

impl Default for Offset {
    /// Nothing, forwards.
    ///
    /// The direction has to start somewhere and the button shows which way it
    /// is; forwards reads as the plain case, and a camera that was never set
    /// starts in 1980 and needs moving forwards anyway.
    fn default() -> Offset {
        Offset {
            days: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
            forward: true,
        }
    }
}

impl Offset {
    /// The offset as one number of seconds, signed.
    pub fn total_seconds(self) -> i64 {
        let magnitude = self.days.abs() * 86_400
            + self.hours.abs() * 3600
            + self.minutes.abs() * 60
            + self.seconds.abs();

        if self.forward {
            magnitude
        } else {
            -magnitude
        }
    }

    pub fn is_zero(self) -> bool {
        self.total_seconds() == 0
    }

    /// How it reads in the interface: `+1 h 30 min`.
    pub fn describe(self) -> String {
        if self.is_zero() {
            return "no change".to_string();
        }

        let sign = if self.forward { "+" } else { "−" };
        let parts = [
            (self.days.abs(), "d"),
            (self.hours.abs(), "h"),
            (self.minutes.abs(), "min"),
            (self.seconds.abs(), "s"),
        ];

        let written: Vec<String> = parts
            .iter()
            .filter(|(amount, _)| *amount > 0)
            .map(|(amount, unit)| format!("{amount} {unit}"))
            .collect();

        format!("{sign}{}", written.join(" "))
    }
}

/// One file, the timestamps in it, and what they would become.
#[derive(Debug, Clone)]
pub struct Planned {
    pub path: PathBuf,
    /// The fields that would move, in the order they are shown.
    pub fields: Vec<DateField>,
    /// What the capture time becomes, for the preview column.
    pub before: Option<Timestamp>,
    pub after: Option<Timestamp>,
}

impl Planned {
    /// Whether this file has anything to change.
    pub fn changes(&self) -> bool {
        !self.fields.is_empty()
    }
}

/// Works out what each entry's timestamps become.
///
/// Reads nothing: the scan already found the dates, so a folder previews as
/// fast as it can be drawn however many files it holds.
///
/// `chosen` is empty until the user has picked, and an empty choice means
/// every field the file has: the common case is "the camera was wrong, fix
/// everything", and making that the default costs nothing.
pub fn plan(entries: &[Entry], chosen: &BTreeSet<String>, offset: Offset) -> Vec<Planned> {
    let seconds = offset.total_seconds();

    entries
        .iter()
        .map(|entry| planned_for(entry, chosen, seconds))
        .collect()
}

/// Every field name found across the whole selection, for the checkbox list.
pub fn available_fields(entries: &[Entry]) -> Vec<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();

    for entry in entries {
        for field in &entry.dates {
            names.insert(field.name.to_string());
        }
    }

    names.into_iter().collect()
}

/// Applies a plan, rewriting each file in turn.
///
/// Each file is read again and its timestamps located again rather than
/// trusting the ones on screen: the preview may have been sitting there while
/// something else wrote to the folder, and writing to a stale offset would put
/// a date in the middle of the pixels.
pub fn apply(planned: &[Planned], chosen: &BTreeSet<String>, offset: Offset) -> Outcome {
    let seconds = offset.total_seconds();
    let mut outcome = Outcome::default();

    if seconds == 0 {
        return outcome;
    }

    for plan in planned.iter().filter(|plan| plan.changes()) {
        match shift_file(&plan.path, chosen, seconds) {
            Ok(0) => {}
            Ok(changed) => outcome.changed.push((plan.path.clone(), changed)),
            Err(e) => outcome.failed.push((plan.path.clone(), e.to_string())),
        }
    }

    outcome
}

/// What an applied plan did.
#[derive(Debug, Default)]
pub struct Outcome {
    /// The files that were rewritten, and how many timestamps moved in each.
    pub changed: Vec<(PathBuf, usize)>,
    pub failed: Vec<(PathBuf, String)>,
}

impl Outcome {
    pub fn summary(&self) -> String {
        let timestamps: usize = self.changed.iter().map(|(_, count)| count).sum();

        match (self.changed.len(), self.failed.len()) {
            (0, 0) => "Nothing to change".to_string(),
            (files, 0) => format!("Moved {timestamps} timestamp(s) in {files} file(s)"),
            (0, failed) => format!("{failed} file(s) could not be changed"),
            (files, failed) => format!("Changed {files}, {failed} could not be"),
        }
    }
}

fn planned_for(entry: &Entry, chosen: &BTreeSet<String>, seconds: i64) -> Planned {
    let before = entry
        .dates
        .iter()
        .find(|field| field.name == super::CAPTURE_TAG)
        .map(|field| field.value);

    let fields: Vec<DateField> = entry
        .dates
        .iter()
        .filter(|field| wanted(chosen, field))
        .cloned()
        .collect();

    let shown = fields
        .iter()
        .find(|field| field.name == super::CAPTURE_TAG)
        .map(|field| field.value);

    Planned {
        path: entry.path.clone(),
        fields,
        before,
        after: shown.map(|value| value.shifted(seconds)),
    }
}

/// Whether a field is one the user asked to move.
fn wanted(chosen: &BTreeSet<String>, field: &DateField) -> bool {
    chosen.is_empty() || chosen.contains(field.name)
}

/// Rewrites one file's timestamps.
///
/// The file is read, changed in memory, and written back whole. Patching it in
/// place would be faster and would leave a half written file behind if the
/// machine stopped in the middle of it.
fn shift_file(path: &Path, chosen: &BTreeSet<String>, seconds: i64) -> std::io::Result<usize> {
    let mut data = std::fs::read(path)?;

    let fields: Vec<DateField> = dates::fields(&data, Format::from_path(path))
        .into_iter()
        .filter(|field| wanted(chosen, field))
        .collect();

    let changed = dates::shift(&mut data, &fields, seconds);
    if changed == 0 {
        return Ok(0);
    }

    write_atomically(path, &data)?;

    Ok(changed)
}

/// Writes `data` over `path` without ever leaving it half written.
///
/// The new copy goes beside the old one and is renamed over it, which the
/// filesystem does as one step.
fn write_atomically(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension(format!(
        "{}.avis-tmp",
        path.extension()
            .map(|ext| ext.to_string_lossy().into_owned())
            .unwrap_or_default()
    ));

    std::fs::write(&temporary, data)?;

    if let Err(e) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::tags;
    use crate::metadata::tiff::test_support::build_tiff_with_sub_ifd;
    use crate::metadata::value::FieldType;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-timeshift-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// A JPEG carrying a capture time and a modification time.
    fn photo(dir: &Path, name: &str, original: &str, modified: &str) -> PathBuf {
        let ascii = |text: &str| {
            let mut bytes = text.as_bytes().to_vec();
            bytes.push(0);
            bytes
        };

        let root = vec![(0x0132u16, FieldType::Ascii, 20, ascii(modified))];
        let exif = vec![(0x9003u16, FieldType::Ascii, 20, ascii(original))];
        let block = build_tiff_with_sub_ifd(&root, tags::EXIF_IFD_POINTER, &exif);

        let mut payload = b"Exif\0\0".to_vec();
        payload.extend_from_slice(&block);

        let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
        out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(&payload);
        out.extend_from_slice(&[0xFF, 0xDA, 0, 2, 0, 0, 0, 0]);

        let path = dir.join(name);
        std::fs::write(&path, out).unwrap();

        path
    }

    /// Entries as the folder scan would have left them: the dates read out of
    /// the front of each file.
    fn entries_at(paths: &[PathBuf]) -> Vec<Entry> {
        paths
            .iter()
            .map(|path| Entry {
                path: path.clone(),
                dates: read_fields(path),
                ..Entry::new(path.clone())
            })
            .collect()
    }

    fn read_fields(path: &Path) -> Vec<DateField> {
        let Ok(data) = std::fs::read(path) else {
            return Vec::new();
        };

        dates::fields(&data, Format::from_path(path))
    }

    fn capture_of(path: &Path) -> String {
        read_fields(path)
            .into_iter()
            .find(|field| field.name == super::super::CAPTURE_TAG)
            .map(|field| field.value.to_exif())
            .unwrap_or_default()
    }

    fn forward(hours: i64) -> Offset {
        Offset {
            hours,
            forward: true,
            ..Default::default()
        }
    }

    #[test]
    fn an_offset_adds_up_its_parts() {
        let offset = Offset {
            days: 1,
            hours: 2,
            minutes: 3,
            seconds: 4,
            forward: true,
        };

        assert_eq!(offset.total_seconds(), 86_400 + 7200 + 180 + 4);
    }

    #[test]
    fn going_back_is_the_same_amount_the_other_way() {
        let back = Offset {
            hours: 2,
            forward: false,
            ..Default::default()
        };

        assert_eq!(back.total_seconds(), -7200);
    }

    #[test]
    fn a_negative_box_does_not_flip_the_direction_twice() {
        // The direction is a toggle, so a negative typed into a box is a slip
        // rather than a second minus sign.
        let offset = Offset {
            hours: -2,
            forward: false,
            ..Default::default()
        };

        assert_eq!(offset.total_seconds(), -7200);
    }

    #[test]
    fn nothing_at_all_is_the_default_and_it_reads_forwards() {
        let offset = Offset::default();

        assert!(offset.is_zero());
        assert!(offset.forward, "so the button reads Forward, not Back");
    }

    #[test]
    fn an_offset_reads_as_a_sentence() {
        assert_eq!(forward(1).describe(), "+1 h");
        assert_eq!(Offset::default().describe(), "no change");
        assert_eq!(
            Offset {
                days: 2,
                minutes: 30,
                forward: false,
                ..Default::default()
            }
            .describe(),
            "−2 d 30 min"
        );
    }

    #[test]
    fn a_plan_shows_what_each_file_becomes() {
        let dir = temp_dir("plan");
        let path = photo(&dir, "a.jpg", "2024:11:06 22:07:19", "2024:11:07 09:00:00");

        let planned = plan(
            &entries_at(std::slice::from_ref(&path)),
            &BTreeSet::new(),
            forward(1),
        );

        assert_eq!(planned[0].before.unwrap().to_exif(), "2024:11:06 22:07:19");
        assert_eq!(planned[0].after.unwrap().to_exif(), "2024:11:06 23:07:19");
        assert_eq!(planned[0].fields.len(), 2, "both dates by default");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn applying_moves_the_dates_in_the_file() {
        let dir = temp_dir("apply");
        let path = photo(&dir, "a.jpg", "2024:11:06 22:07:19", "2024:11:07 09:00:00");

        let planned = plan(
            &entries_at(std::slice::from_ref(&path)),
            &BTreeSet::new(),
            forward(1),
        );
        let outcome = apply(&planned, &BTreeSet::new(), forward(1));

        assert_eq!(outcome.failed.len(), 0);
        assert_eq!(outcome.changed, vec![(path.clone(), 2)]);
        assert_eq!(capture_of(&path), "2024:11:06 23:07:19");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_the_chosen_fields_are_offered_to_be_moved() {
        let dir = temp_dir("chosen");
        let path = photo(&dir, "a.jpg", "2024:11:06 22:07:19", "2024:11:07 09:00:00");

        let mut chosen = BTreeSet::new();
        chosen.insert(super::super::CAPTURE_TAG.to_string());

        let planned = plan(
            &entries_at(std::slice::from_ref(&path)),
            &chosen,
            forward(1),
        );
        assert_eq!(planned[0].fields.len(), 1);

        apply(&planned, &chosen, forward(1));

        let after = read_fields(&path);
        let modified = after
            .iter()
            .find(|field| field.name == "Modify Date")
            .unwrap();

        assert_eq!(capture_of(&path), "2024:11:06 23:07:19");
        assert_eq!(modified.value.to_exif(), "2024:11:07 09:00:00", "untouched");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_fields_a_folder_offers_are_the_union_of_what_it_has() {
        let dir = temp_dir("fields");
        let paths = vec![
            photo(&dir, "a.jpg", "2024:11:06 22:07:19", "2024:11:07 09:00:00"),
            photo(&dir, "b.jpg", "2024:11:06 22:08:00", "2024:11:07 09:00:00"),
        ];

        assert_eq!(
            available_fields(&entries_at(&paths)),
            vec!["Date/Time Original".to_string(), "Modify Date".to_string()]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_shift_of_nothing_touches_no_file() {
        let dir = temp_dir("zero");
        let path = photo(&dir, "a.jpg", "2024:11:06 22:07:19", "2024:11:07 09:00:00");
        let before = std::fs::read(&path).unwrap();

        let planned = plan(
            &entries_at(std::slice::from_ref(&path)),
            &BTreeSet::new(),
            Offset::default(),
        );
        let outcome = apply(&planned, &BTreeSet::new(), Offset::default());

        assert!(outcome.changed.is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), before);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_shift_and_its_opposite_leave_the_file_as_it_was() {
        let dir = temp_dir("undo");
        let path = photo(&dir, "a.jpg", "2024:11:06 22:07:19", "2024:11:07 09:00:00");
        let before = std::fs::read(&path).unwrap();

        let entries = entries_at(std::slice::from_ref(&path));
        let back = Offset {
            hours: 3,
            forward: false,
            ..Default::default()
        };

        apply(
            &plan(&entries, &BTreeSet::new(), forward(3)),
            &BTreeSet::new(),
            forward(3),
        );
        apply(
            &plan(&entries, &BTreeSet::new(), back),
            &BTreeSet::new(),
            back,
        );

        assert_eq!(std::fs::read(&path).unwrap(), before, "byte for byte");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_with_no_dates_is_left_out_of_the_plan() {
        let dir = temp_dir("nodates");
        let path = dir.join("notes.txt");
        std::fs::write(&path, b"nothing to see").unwrap();

        let planned = plan(
            &entries_at(std::slice::from_ref(&path)),
            &BTreeSet::new(),
            forward(1),
        );

        assert!(!planned[0].changes());
        assert!(planned[0].before.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_that_is_not_there_is_reported_rather_than_panicked_on() {
        let planned = plan(
            &entries_at(&[PathBuf::from("does-not-exist.jpg")]),
            &BTreeSet::new(),
            forward(1),
        );

        assert!(!planned[0].changes());
        assert!(apply(&planned, &BTreeSet::new(), forward(1))
            .failed
            .is_empty());
    }

    #[test]
    fn nothing_is_left_behind_when_a_file_is_written() {
        let dir = temp_dir("clean");
        let path = photo(&dir, "a.jpg", "2024:11:06 22:07:19", "2024:11:07 09:00:00");

        let planned = plan(
            &entries_at(std::slice::from_ref(&path)),
            &BTreeSet::new(),
            forward(1),
        );
        apply(&planned, &BTreeSet::new(), forward(1));

        let left: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();

        assert_eq!(left.len(), 1, "no temporary file survived: {left:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}

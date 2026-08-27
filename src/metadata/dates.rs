//! Finding the timestamps in a file, and moving them.
//!
//! A camera with the wrong clock stamps every photograph of a trip with the
//! wrong time, and the fix is arithmetic: add or subtract the same offset from
//! every date the file carries.
//!
//! It is done by rewriting the bytes where they already are rather than by
//! rebuilding the file. An EXIF timestamp is a fixed nineteen ASCII characters
//! whatever it says, so a shifted one is exactly as long as the one it
//! replaces — which means nothing else in the file has to move, no offset has
//! to be recomputed, and the maker notes, the thumbnail and the pixels are all
//! left untouched. Rebuilding a raw file to change a date would be a great
//! deal of risk for no gain.

use crate::formats::Format;
use crate::metadata::containers::{self, ExifBlock};
use crate::metadata::datetime::{Timestamp, EXIF_LEN};
use crate::metadata::tags::{self, IfdKind};
use crate::metadata::tiff::{Ifd, Tiff};
use crate::metadata::value::FieldType;

/// The tags that hold a date, and where each of them lives.
///
/// Deliberately a list rather than "every ASCII field that parses as a date":
/// a maker note or a caption could easily contain one, and rewriting those
/// would be a surprise.
const DATE_TAGS: &[(IfdKind, u16)] = &[
    // When the shutter opened. The one that matters.
    (IfdKind::Exif, 0x9003),
    // When the file was written, which for a camera is the same moment.
    (IfdKind::Exif, 0x9004),
    // The file's own modification date, in IFD0.
    (IfdKind::Root, 0x0132),
    // Some cameras repeat the original in IFD0 as well.
    (IfdKind::Root, 0x9003),
    (IfdKind::Root, 0x9004),
];

/// One timestamp in a file: what it is called, what it says, and where the
/// characters that say it begin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateField {
    /// The EXIF name of the tag, as the side panel would show it.
    pub name: &'static str,
    /// Byte offset of the value within the file.
    pub offset: usize,
    pub value: Timestamp,
}

/// Every timestamp `data` carries that can be rewritten in place.
///
/// Sorted by name so a list of them is stable, and deduplicated by offset
/// because a file can reach the same directory by more than one path.
pub fn fields(data: &[u8], format: Option<Format>) -> Vec<DateField> {
    let extracted = containers::extract(data, format);
    let mut found: Vec<DateField> = Vec::new();

    for block in &extracted.exif {
        let Some(base) = offset_within(data, block.bytes) else {
            continue;
        };

        collect(block, base, &mut found);
    }

    found.sort_by(|a, b| a.name.cmp(b.name).then(a.offset.cmp(&b.offset)));
    found.dedup_by_key(|field| field.offset);

    found
}

/// Rewrites `fields` in `data`, moved by `seconds`.
///
/// Returns how many were changed. A field whose bytes no longer say what they
/// said when it was found is left alone: the caller may have been sitting on
/// its list while something else wrote to the file.
pub fn shift(data: &mut [u8], fields: &[DateField], seconds: i64) -> usize {
    let mut changed = 0;

    for field in fields {
        let Some(existing) = data.get(field.offset..field.offset + EXIF_LEN) else {
            continue;
        };

        if std::str::from_utf8(existing)
            .ok()
            .and_then(Timestamp::parse)
            != Some(field.value)
        {
            tracing::warn!("{} moved or changed; leaving it alone", field.name);
            continue;
        }

        let shifted = field.value.shifted(seconds).to_exif();
        if shifted.len() != EXIF_LEN {
            continue;
        }

        data[field.offset..field.offset + EXIF_LEN].copy_from_slice(shifted.as_bytes());
        changed += 1;
    }

    changed
}

/// The timestamps in one TIFF block, with their offsets made absolute.
fn collect(block: &ExifBlock<'_>, base: usize, found: &mut Vec<DateField>) {
    let Some(tiff) = Tiff::new(block.bytes) else {
        return;
    };

    for (kind, ifd) in directories(&tiff, block.kind) {
        for (wanted_kind, tag) in DATE_TAGS {
            if *wanted_kind != kind {
                continue;
            }

            if let Some(field) = read(&tiff, &ifd, kind, *tag, base) {
                found.push(field);
            }
        }
    }
}

/// One tag, if it is there and holds a timestamp we can rewrite.
fn read(tiff: &Tiff<'_>, ifd: &Ifd, kind: IfdKind, tag: u16, base: usize) -> Option<DateField> {
    let entry = ifd.entry(tag)?;

    // Anything else is a camera doing something unusual, and not something to
    // overwrite on a guess.
    if entry.field_type != FieldType::Ascii || entry.byte_len() < EXIF_LEN {
        return None;
    }

    let bytes = tiff.entry_bytes(entry)?;
    let value = Timestamp::parse(std::str::from_utf8(bytes).ok()?)?;

    Some(DateField {
        name: tags::name(kind, tag)?,
        offset: base + entry.offset,
        value,
    })
}

/// The root directory and the EXIF sub-directory it points at.
///
/// Only these two: the dates live in them, and walking further would reach the
/// thumbnail's own directory, whose modification date is not the photograph's.
fn directories(tiff: &Tiff<'_>, kind: IfdKind) -> Vec<(IfdKind, Ifd)> {
    let Some(root) = tiff.ifds().into_iter().next() else {
        return Vec::new();
    };

    // A CR3 stores each directory as its own block, so the block already says
    // what it holds and there is nothing to follow.
    if kind != IfdKind::Root {
        return vec![(kind, root)];
    }

    let exif = root
        .u32(tags::EXIF_IFD_POINTER)
        .and_then(|offset| tiff.ifd_at(offset))
        .map(|ifd| (IfdKind::Exif, ifd));

    let mut all = vec![(IfdKind::Root, root)];
    all.extend(exif);

    all
}

/// Where `part` starts inside `whole`, when it is a slice of it.
///
/// Address arithmetic because that is what the question is: the container
/// walkers hand back sub-slices of the file buffer, and rewriting one means
/// knowing where in the file it came from. Searching for the bytes instead
/// would find the wrong copy in a file with two similar blocks.
fn offset_within(whole: &[u8], part: &[u8]) -> Option<usize> {
    let start = whole.as_ptr() as usize;
    let inner = part.as_ptr() as usize;

    let offset = inner.checked_sub(start)?;
    (offset + part.len() <= whole.len()).then_some(offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::tiff::test_support::build_tiff_with_sub_ifd;

    /// A JPEG whose EXIF holds the two dates a camera writes.
    fn jpeg_with_dates(original: &str, modified: &str) -> Vec<u8> {
        let ascii = |text: &str| {
            let mut bytes = text.as_bytes().to_vec();
            bytes.push(0);
            bytes
        };

        let root = vec![(
            0x0132u16,
            FieldType::Ascii,
            (modified.len() + 1) as u32,
            ascii(modified),
        )];
        let exif = vec![(
            0x9003u16,
            FieldType::Ascii,
            (original.len() + 1) as u32,
            ascii(original),
        )];

        let block = build_tiff_with_sub_ifd(&root, tags::EXIF_IFD_POINTER, &exif);

        let mut payload = b"Exif\0\0".to_vec();
        payload.extend_from_slice(&block);

        let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
        out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(&payload);
        out.extend_from_slice(&[0xFF, 0xDA, 0, 2, 0, 0, 0, 0]);

        out
    }

    fn names(fields: &[DateField]) -> Vec<&str> {
        fields.iter().map(|field| field.name).collect()
    }

    #[test]
    fn finds_the_dates_a_camera_writes() {
        let data = jpeg_with_dates("2024:11:06 22:07:19", "2024:11:07 09:00:00");
        let found = fields(&data, Some(Format::Jpeg));

        assert_eq!(names(&found), vec!["Date/Time Original", "Modify Date"]);
        assert_eq!(
            found[0].value,
            Timestamp::parse("2024:11:06 22:07:19").unwrap()
        );
    }

    #[test]
    fn the_offsets_point_at_the_characters_themselves() {
        let data = jpeg_with_dates("2024:11:06 22:07:19", "2024:11:07 09:00:00");

        for field in fields(&data, Some(Format::Jpeg)) {
            let bytes = &data[field.offset..field.offset + EXIF_LEN];
            assert_eq!(std::str::from_utf8(bytes).unwrap(), field.value.to_exif());
        }
    }

    #[test]
    fn shifting_moves_every_date_by_the_same_amount() {
        let mut data = jpeg_with_dates("2024:11:06 22:07:19", "2024:11:07 09:00:00");
        let found = fields(&data, Some(Format::Jpeg));

        // An hour back, as if the camera had been left on summer time.
        assert_eq!(shift(&mut data, &found, -3600), 2);

        let after = fields(&data, Some(Format::Jpeg));
        assert_eq!(after[0].value.to_exif(), "2024:11:06 21:07:19");
        assert_eq!(after[1].value.to_exif(), "2024:11:07 08:00:00");
    }

    #[test]
    fn a_shift_can_be_undone_exactly() {
        let original = jpeg_with_dates("2024:11:06 22:07:19", "2024:11:07 09:00:00");
        let mut data = original.clone();

        let offset = 3 * 86_400 + 7 * 3600 + 42;
        let found = fields(&data, Some(Format::Jpeg));
        shift(&mut data, &found, offset);

        let found = fields(&data, Some(Format::Jpeg));
        shift(&mut data, &found, -offset);

        assert_eq!(data, original, "the file came back byte for byte");
    }

    #[test]
    fn only_the_chosen_fields_move() {
        let mut data = jpeg_with_dates("2024:11:06 22:07:19", "2024:11:07 09:00:00");
        let found = fields(&data, Some(Format::Jpeg));
        let original: Vec<DateField> = found
            .iter()
            .filter(|field| field.name == "Date/Time Original")
            .cloned()
            .collect();

        assert_eq!(shift(&mut data, &original, 3600), 1);

        let after = fields(&data, Some(Format::Jpeg));
        assert_eq!(after[0].value.to_exif(), "2024:11:06 23:07:19");
        assert_eq!(after[1].value.to_exif(), "2024:11:07 09:00:00", "untouched");
    }

    #[test]
    fn nothing_but_the_dates_changes() {
        let before = jpeg_with_dates("2024:11:06 22:07:19", "2024:11:07 09:00:00");
        let mut after = before.clone();
        let found = fields(&after, Some(Format::Jpeg));

        shift(&mut after, &found, 3600);

        assert_eq!(before.len(), after.len());
        let differing = before.iter().zip(&after).filter(|(a, b)| a != b).count();

        // One hour later differs in the two hour digits of each date.
        assert!(differing <= 4, "{differing} bytes changed");
    }

    #[test]
    fn a_stale_field_is_left_alone() {
        let mut data = jpeg_with_dates("2024:11:06 22:07:19", "2024:11:07 09:00:00");
        let mut found = fields(&data, Some(Format::Jpeg));

        // As if the file had been rewritten since the list was made.
        found[0].value = Timestamp::parse("1999:01:01 00:00:00").unwrap();

        assert_eq!(shift(&mut data, &found, 3600), 1);
        assert_eq!(
            fields(&data, Some(Format::Jpeg))[0].value.to_exif(),
            "2024:11:06 22:07:19"
        );
    }

    #[test]
    fn an_offset_past_the_end_is_ignored_rather_than_panicked_on() {
        let mut data = jpeg_with_dates("2024:11:06 22:07:19", "2024:11:07 09:00:00");
        let stale = vec![DateField {
            name: "Date/Time Original",
            offset: data.len() + 100,
            value: Timestamp::parse("2024:11:06 22:07:19").unwrap(),
        }];

        assert_eq!(shift(&mut data, &stale, 3600), 0);
    }

    #[test]
    fn a_file_with_no_metadata_has_no_dates() {
        assert!(fields(b"not an image at all", None).is_empty());
        assert!(fields(&[], None).is_empty());
    }

    #[test]
    fn a_date_a_camera_never_set_is_not_offered() {
        let data = jpeg_with_dates("0000:00:00 00:00:00", "2024:11:07 09:00:00");
        assert_eq!(
            names(&fields(&data, Some(Format::Jpeg))),
            vec!["Modify Date"]
        );
    }

    #[test]
    fn a_slice_of_a_buffer_knows_where_it_came_from() {
        let whole = vec![0u8; 100];

        assert_eq!(offset_within(&whole, &whole[10..20]), Some(10));
        assert_eq!(offset_within(&whole, &whole[..]), Some(0));
        assert_eq!(offset_within(&whole, &[1, 2, 3]), None);
    }
}

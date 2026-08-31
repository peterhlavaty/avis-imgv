//! Camera raw containers.
//!
//! Raw files are not decoded: the viewer shows the full size JPEG preview that
//! cameras embed, which is what raw converters display before a develop step
//! and is orders of magnitude cheaper to produce.

use std::collections::BTreeSet;

use super::{ExifBlock, Extracted, Pixels};
use crate::metadata::tags;
use crate::metadata::tiff::{Ifd, Tiff};

const RAF_MAGIC: &[u8] = b"FUJIFILMCCD-RAW";
const RAF_JPEG_OFFSET: usize = 84;
const SOI: [u8; 3] = [0xFF, 0xD8, 0xFF];

/// Previews smaller than this are thumbnails, not something worth displaying.
const MIN_PREVIEW_BYTES: usize = 8 * 1024;

/// Reads the EXIF block, ICC profile and largest embedded preview of a raw
/// file.
pub fn extract(data: &[u8]) -> Extracted<'_> {
    if data.starts_with(RAF_MAGIC) {
        return raf(data);
    }

    match Tiff::new(data) {
        Some(tiff) => tiff_based(&tiff, data),
        // A vendor container we do not know. Scanning for a preview is the
        // last resort, and better than showing nothing.
        None => Extracted {
            preview: largest_embedded_jpeg(data),
            ..Default::default()
        },
    }
}

/// The small JPEG a camera files in IFD1, which is what a thumbnail is.
///
/// Every directory is looked at rather than only IFD1, because raw files put
/// theirs wherever they like; the smallest candidate is the thumbnail and the
/// largest is the preview.
pub fn thumbnail<'a>(tiff: &Tiff<'a>) -> Option<&'a [u8]> {
    directories(tiff)
        .iter()
        .flat_map(|ifd| jpeg_candidates(tiff, ifd))
        .min_by_key(|candidate| candidate.len())
}

/// Extracts from a TIFF derived raw (DNG, NEF, CR2, ARW, ORF, ...).
fn tiff_based<'a>(tiff: &Tiff<'a>, data: &'a [u8]) -> Extracted<'a> {
    let mut best: Option<&'a [u8]> = None;
    let mut icc = None;

    let all = directories(tiff);

    for ifd in &all {
        if icc.is_none() {
            icc = ifd
                .entry(tags::ICC_PROFILE)
                .and_then(|e| tiff.entry_bytes(e))
                .map(<[u8]>::to_vec);
        }

        for candidate in jpeg_candidates(tiff, ifd) {
            if best.is_none_or(|current| candidate.len() > current.len()) {
                best = Some(candidate);
            }
        }
    }

    let preview = best.filter(|p| p.len() >= MIN_PREVIEW_BYTES);

    Extracted {
        exif: vec![ExifBlock::root(data)],
        icc,
        // Only looked for when there is no JPEG one: it is always the small
        // reduced-resolution copy, and decoding it is a last resort.
        pixels: preview
            .is_none()
            .then(|| pixel_preview(tiff, &all))
            .flatten(),
        preview,
        thumbnail: thumbnail(tiff),
        full_size: main_image_size(&all),
        // The packet lives in a tag, which the directory walk picks up.
        xmp: None,
    }
}

/// The dimensions of the photograph itself, rather than of a copy of it.
///
/// A raw file's first directory is very often the reduced-resolution preview
/// — for a DNG written by Camera Raw it is a 256 pixel thumbnail — and the
/// sensor data sits in a sub-directory. Reading the size off the first one
/// meant the side panel reported `256x171` for a forty-five megapixel
/// photograph, and the viewer drew a preview at "100%" that was a postage
/// stamp in the middle of the screen.
///
/// `NewSubfileType` names the main image explicitly: bit 0 clear means "not a
/// reduced resolution copy". Where nothing says, the largest wins.
fn main_image_size(all: &[Ifd]) -> Option<(u32, u32)> {
    let sized = |ifd: &Ifd| {
        let width = ifd.u32(tags::IMAGE_WIDTH)?;
        let height = ifd.u32(tags::IMAGE_HEIGHT)?;
        (width > 0 && height > 0).then_some((width, height))
    };

    let full = |ifd: &&Ifd| {
        ifd.u32(tags::NEW_SUBFILE_TYPE)
            .is_some_and(|kind| kind & 1 == 0)
    };

    all.iter()
        .filter(full)
        .filter_map(sized)
        .chain(all.iter().filter_map(sized))
        .max_by_key(|(width, height)| u64::from(*width) * u64::from(*height))
}

/// An uncompressed preview, for the raws that embed no JPEG at all.
///
/// A DNG written by Camera Raw carries its reduced-resolution copy as plain
/// eight-bit RGB strips rather than as a JPEG, so the scan for embedded JPEGs
/// found nothing and the file fell through to the `image` crate — which
/// decodes the *first* directory, which is that same small copy, and reported
/// it as the photograph.
fn pixel_preview<'a>(tiff: &Tiff<'a>, all: &[Ifd]) -> Option<Pixels<'a>> {
    all.iter()
        .filter_map(|ifd| uncompressed(tiff, ifd))
        .max_by_key(|found| u64::from(found.width) * u64::from(found.height))
}

/// The uncompressed picture a directory points at, if that is what it holds.
fn uncompressed<'a>(tiff: &Tiff<'a>, ifd: &Ifd) -> Option<Pixels<'a>> {
    if ifd.u32(tags::COMPRESSION) != Some(1) {
        return None;
    }

    let width = ifd.u32(tags::IMAGE_WIDTH)?;
    let height = ifd.u32(tags::IMAGE_HEIGHT)?;
    let samples = ifd.u32(tags::SAMPLES_PER_PIXEL).unwrap_or(1);

    // Eight bits a channel, and either grey or RGB. Anything else is sensor
    // data rather than a picture of one.
    if ifd.u32(tags::BITS_PER_SAMPLE) != Some(8) || !matches!(samples, 1 | 3) {
        return None;
    }

    let wanted = u64::from(width) * u64::from(height) * u64::from(samples);
    let offset = ifd.u32(tags::STRIP_OFFSETS)? as usize;
    let length = ifd.u32(tags::STRIP_BYTE_COUNTS)? as usize;

    // One strip holding the whole thing: a reduced-resolution copy is small
    // enough that every writer does it that way, and stitching several would
    // be work for a picture nobody wants to look at anyway.
    if length as u64 != wanted {
        return None;
    }

    Some(Pixels {
        bytes: tiff.bytes(offset, length)?,
        width,
        height,
        samples: samples as u8,
    })
}

/// The top level directories plus one level of sub-directories, which is where
/// every raw format we support hides its previews.
///
/// The sub-directory offsets come out of the file, so a crafted one can list a
/// thousand of them all pointing at the same fat directory. Reading it a
/// thousand times cost seconds and gigabytes on the preview thread, which is
/// the thread that has to answer in milliseconds; each offset is now read once
/// and the total is capped, the way every other walk in this module is.
fn directories(tiff: &Tiff<'_>) -> Vec<Ifd> {
    let mut all = tiff.ifds();

    let sub_offsets: BTreeSet<u32> = all
        .iter()
        .flat_map(|ifd| ifd.u32_list(tags::SUB_IFDS))
        .collect();

    for offset in sub_offsets {
        if all.len() >= MAX_DIRECTORIES {
            tracing::debug!("Stopping at {MAX_DIRECTORIES} directories");
            break;
        }

        if let Some(ifd) = tiff.ifd_at(offset) {
            all.push(ifd);
        }
    }

    all
}

/// How many directories are walked before a file is assumed to be lying.
///
/// The same order as the chain limit in [`crate::metadata::tiff`]: a raw file
/// files its previews in a handful of them, and anything claiming hundreds is
/// not a photograph.
const MAX_DIRECTORIES: usize = 64;

/// JPEG blobs referenced by a directory, either as a thumbnail pair or as JPEG
/// compressed strips.
fn jpeg_candidates<'a>(tiff: &Tiff<'a>, ifd: &Ifd) -> Vec<&'a [u8]> {
    let pairs = [
        (
            tags::JPEG_INTERCHANGE_FORMAT,
            tags::JPEG_INTERCHANGE_FORMAT_LENGTH,
        ),
        (tags::STRIP_OFFSETS, tags::STRIP_BYTE_COUNTS),
    ];

    pairs
        .iter()
        .filter_map(|(offset_tag, length_tag)| {
            let offset = ifd.u32(*offset_tag)? as usize;
            let length = ifd.u32(*length_tag)? as usize;
            tiff.bytes(offset, length)
        })
        .filter(|bytes| bytes.starts_with(&SOI))
        .collect()
}

/// A Fuji raw, whose EXIF lives inside the JPEG it embeds.
///
/// The header points at a perfectly ordinary JPEG, and that JPEG's own APP1
/// carries the directories — so the file's metadata is read by reading its
/// preview. This used to be unpacked by the one caller that noticed, which
/// left every other caller with nothing: the capture-time shift, which asks
/// the container directly, found no timestamps in a RAF at all and silently
/// declined to move a Fujifilm shoot's clock.
///
/// The preview is a subslice of the file rather than a copy, so the offsets
/// the shift needs are the file's own offsets and the timestamps can be
/// rewritten where they lie.
fn raf(data: &[u8]) -> Extracted<'_> {
    let Some(preview) = raf_preview(data) else {
        return Extracted::default();
    };

    let inner = super::jpeg::extract(preview);

    Extracted {
        preview: Some(preview),
        exif: inner.exif,
        icc: inner.icc,
        xmp: inner.xmp,
        // The embedded JPEG carries a thumbnail of its own, which is what
        // puts a Fuji raw on the contact sheet as fast as a JPEG.
        thumbnail: inner.thumbnail,
        ..Default::default()
    }
}

/// Fuji raws point at their preview from a fixed header offset.
fn raf_preview(data: &[u8]) -> Option<&[u8]> {
    let read_u32 = |at: usize| -> Option<usize> {
        let b = data.get(at..at + 4)?;
        Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
    };

    let offset = read_u32(RAF_JPEG_OFFSET)?;
    let length = read_u32(RAF_JPEG_OFFSET + 4)?;

    data.get(offset..offset.checked_add(length)?)
        .filter(|bytes| bytes.starts_with(&SOI))
}

/// Finds the biggest JPEG stream in an opaque container.
///
/// Streams are delimited by their start markers; the largest span between two
/// consecutive markers is the full size preview rather than a thumbnail. The
/// marker after the start of image must be one a real encoder writes, or
/// compressed sensor data would pass for a picture.
fn largest_embedded_jpeg(data: &[u8]) -> Option<&[u8]> {
    let starts: Vec<usize> = data
        .windows(SOI.len() + 1)
        .enumerate()
        .filter(|(_, window)| window[..SOI.len()] == SOI && is_leading_marker(window[SOI.len()]))
        .map(|(i, _)| i)
        .collect();

    starts
        .iter()
        .enumerate()
        .map(|(i, start)| *start..starts.get(i + 1).copied().unwrap_or(data.len()))
        .max_by_key(std::ops::Range::len)
        .map(|range| &data[range])
        .filter(|preview| preview.len() >= MIN_PREVIEW_BYTES)
}

/// Markers a JPEG encoder puts first: a frame header, a quantisation table,
/// an application segment or a comment.
fn is_leading_marker(marker: u8) -> bool {
    matches!(marker, 0xC0..=0xCF | 0xDB | 0xE0..=0xEF | 0xFE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::tiff::test_support::build_tiff;
    use crate::metadata::value::FieldType;

    fn fake_jpeg(size: usize) -> Vec<u8> {
        let mut jpeg = SOI.to_vec();
        jpeg.push(0xE0);
        jpeg.resize(size, 0x5A);
        jpeg
    }

    #[test]
    fn picks_the_largest_preview_from_a_tiff_raw() {
        let thumbnail = fake_jpeg(MIN_PREVIEW_BYTES * 2);
        let preview = fake_jpeg(MIN_PREVIEW_BYTES * 8);

        // Payloads land after the directory, in the order they are declared.
        let mut data = build_tiff(&[
            (tags::JPEG_INTERCHANGE_FORMAT, FieldType::Long, 1, vec![]),
            (
                tags::JPEG_INTERCHANGE_FORMAT_LENGTH,
                FieldType::Long,
                1,
                vec![],
            ),
            (tags::STRIP_OFFSETS, FieldType::Long, 1, vec![]),
            (tags::STRIP_BYTE_COUNTS, FieldType::Long, 1, vec![]),
        ]);

        let thumbnail_at = data.len();
        data.extend_from_slice(&thumbnail);
        let preview_at = data.len();
        data.extend_from_slice(&preview);

        // Patch the four inline values now that the offsets are known.
        let entry_value = |index: usize| 8 + 2 + index * 12 + 8;
        for (index, value) in [thumbnail_at, thumbnail.len(), preview_at, preview.len()]
            .iter()
            .enumerate()
        {
            let at = entry_value(index);
            data[at..at + 4].copy_from_slice(&(*value as u32).to_le_bytes());
        }

        let found = extract(&data);
        assert_eq!(found.preview.map(<[u8]>::len), Some(preview.len()));
        assert!(found.root_exif().is_some());
    }

    #[test]
    fn the_thumbnail_is_the_smaller_of_the_two() {
        let thumbnail = fake_jpeg(MIN_PREVIEW_BYTES * 2);
        let preview = fake_jpeg(MIN_PREVIEW_BYTES * 8);

        let mut data = build_tiff(&[
            (tags::JPEG_INTERCHANGE_FORMAT, FieldType::Long, 1, vec![]),
            (
                tags::JPEG_INTERCHANGE_FORMAT_LENGTH,
                FieldType::Long,
                1,
                vec![],
            ),
            (tags::STRIP_OFFSETS, FieldType::Long, 1, vec![]),
            (tags::STRIP_BYTE_COUNTS, FieldType::Long, 1, vec![]),
        ]);

        let thumbnail_at = data.len();
        data.extend_from_slice(&thumbnail);
        let preview_at = data.len();
        data.extend_from_slice(&preview);

        let entry_value = |index: usize| 8 + 2 + index * 12 + 8;
        for (index, value) in [thumbnail_at, thumbnail.len(), preview_at, preview.len()]
            .iter()
            .enumerate()
        {
            let at = entry_value(index);
            data[at..at + 4].copy_from_slice(&(*value as u32).to_le_bytes());
        }

        let found = extract(&data);

        assert_eq!(found.thumbnail.map(<[u8]>::len), Some(thumbnail.len()));
        assert_eq!(found.preview.map(<[u8]>::len), Some(preview.len()));
    }

    #[test]
    fn reads_the_fuji_header() {
        let preview = fake_jpeg(MIN_PREVIEW_BYTES + 100);
        let mut data = RAF_MAGIC.to_vec();
        data.resize(128, 0);
        let offset = data.len();
        data.extend_from_slice(&preview);

        data[RAF_JPEG_OFFSET..RAF_JPEG_OFFSET + 4].copy_from_slice(&(offset as u32).to_be_bytes());
        data[RAF_JPEG_OFFSET + 4..RAF_JPEG_OFFSET + 8]
            .copy_from_slice(&(preview.len() as u32).to_be_bytes());

        assert_eq!(extract(&data).preview, Some(preview.as_slice()));
    }

    #[test]
    fn scans_opaque_containers_for_the_biggest_stream() {
        let mut data = b"ftypcrx ".to_vec();
        data.extend_from_slice(&fake_jpeg(MIN_PREVIEW_BYTES));
        let big = fake_jpeg(MIN_PREVIEW_BYTES * 4);
        data.extend_from_slice(&big);

        assert_eq!(extract(&data).preview.map(<[u8]>::len), Some(big.len()));
    }

    #[test]
    fn ignores_previews_that_are_only_thumbnails() {
        let mut data = b"ftypcrx ".to_vec();
        data.extend_from_slice(&fake_jpeg(512));
        assert!(extract(&data).preview.is_none());
    }

    #[test]
    fn sensor_data_is_not_mistaken_for_a_preview() {
        let mut data = b"vendor-container".to_vec();
        data.extend_from_slice(&SOI);
        // 0x5A is not a marker any encoder starts a stream with.
        data.extend_from_slice(&vec![0x5Au8; MIN_PREVIEW_BYTES * 4]);

        assert!(extract(&data).preview.is_none());
    }

    #[test]
    fn malformed_input_does_not_panic() {
        for len in 0..64 {
            let _ = extract(&vec![0xFFu8; len]);
        }
        let _ = extract(&RAF_MAGIC[..8]);
    }
}

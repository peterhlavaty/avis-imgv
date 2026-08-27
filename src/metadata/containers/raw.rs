//! Camera raw containers.
//!
//! Raw files are not decoded: the viewer shows the full size JPEG preview that
//! cameras embed, which is what raw converters display before a develop step
//! and is orders of magnitude cheaper to produce.

use super::{ExifBlock, Extracted};
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
        return Extracted {
            preview: raf_preview(data),
            // Fuji stores a regular JPEG whose own APP1 holds the EXIF block.
            ..Default::default()
        };
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

/// Extracts from a TIFF derived raw (DNG, NEF, CR2, ARW, ORF, ...).
fn tiff_based<'a>(tiff: &Tiff<'a>, data: &'a [u8]) -> Extracted<'a> {
    let mut best: Option<&'a [u8]> = None;
    let mut icc = None;

    for ifd in directories(tiff) {
        if icc.is_none() {
            icc = ifd
                .entry(tags::ICC_PROFILE)
                .and_then(|e| tiff.entry_bytes(e))
                .map(<[u8]>::to_vec);
        }

        for candidate in jpeg_candidates(tiff, &ifd) {
            if best.is_none_or(|current| candidate.len() > current.len()) {
                best = Some(candidate);
            }
        }
    }

    Extracted {
        exif: vec![ExifBlock::root(data)],
        icc,
        preview: best.filter(|p| p.len() >= MIN_PREVIEW_BYTES),
        // The packet lives in a tag, which the directory walk picks up.
        xmp: None,
    }
}

/// The top level directories plus one level of sub-directories, which is where
/// every raw format we support hides its previews.
fn directories(tiff: &Tiff<'_>) -> Vec<Ifd> {
    let mut all = tiff.ifds();

    let sub_offsets: Vec<u32> = all
        .iter()
        .flat_map(|ifd| ifd.u32_list(tags::SUB_IFDS))
        .collect();

    all.extend(sub_offsets.iter().filter_map(|o| tiff.ifd_at(*o)));
    all
}

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

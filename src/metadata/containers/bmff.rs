//! ISO base media containers, which is how Canon's CR3 stores a raw file.
//!
//! Unlike the TIFF derived raws there is no IFD chain to follow: the EXIF
//! directories sit in their own boxes, unlinked, and the preview is a JPEG in
//! a box of its own.

use super::{ExifBlock, Extracted};
use crate::metadata::tags::IfdKind;

/// Boxes we descend into looking for metadata.
const CONTAINERS: &[&[u8; 4]] = &[b"moov", b"uuid"];

/// A `uuid` box begins with the 16 byte identifier of its extension.
const UUID_LEN: usize = 16;
const HEADER_LEN: usize = 8;

/// Nesting past this is either a loop or a file we do not understand.
const MAX_DEPTH: usize = 6;

const SOI: [u8; 3] = [0xFF, 0xD8, 0xFF];

/// True when `data` looks like an ISO base media file.
pub fn is_bmff(data: &[u8]) -> bool {
    data.get(4..8) == Some(b"ftyp")
}

/// Collects the EXIF directories and preview of a CR3.
pub fn extract(data: &[u8]) -> Extracted<'_> {
    let mut found = Extracted::default();
    walk(data, 0, &mut found);
    found
}

fn walk<'a>(data: &'a [u8], depth: usize, found: &mut Extracted<'a>) {
    if depth > MAX_DEPTH {
        return;
    }

    for (kind, payload) in Boxes::new(data) {
        // Canon writes the four EXIF directories as separate boxes, each a
        // bare TIFF block, with none of them pointing at the others.
        let directory = match &kind {
            b"CMT1" => Some(IfdKind::Root),
            b"CMT2" => Some(IfdKind::Exif),
            b"CMT4" => Some(IfdKind::Gps),
            _ => None,
        };

        if let Some(kind) = directory {
            found.exif.push(ExifBlock {
                bytes: payload,
                kind,
            });
            continue;
        }

        // PRVW holds the display sized preview, THMB only a thumbnail.
        if matches!(&kind, b"PRVW" | b"THMB") {
            let preview = embedded_jpeg(payload);
            if preview.map(<[u8]>::len) > found.preview.map(<[u8]>::len) {
                found.preview = preview;
            }
            continue;
        }

        if CONTAINERS.contains(&&kind) {
            let inner = if &kind == b"uuid" {
                uuid_body(payload)
            } else {
                Some(payload)
            };

            if let Some(inner) = inner {
                walk(inner, depth + 1, found);
            }
        }
    }
}

/// The boxes inside a `uuid` extension, skipping its identifier.
///
/// Canon's preview extension inserts eight further bytes before its first box,
/// so the body is located by looking for something that parses as one rather
/// than by trusting a fixed offset.
fn uuid_body(payload: &[u8]) -> Option<&[u8]> {
    const CANON_PREVIEW_PAD: usize = 8;

    let body = payload.get(UUID_LEN..)?;
    if starts_with_box(body) {
        return Some(body);
    }

    let padded = body.get(CANON_PREVIEW_PAD..)?;
    starts_with_box(padded).then_some(padded)
}

/// Whether `data` begins with something shaped like a box: a plausible size
/// and a four character type.
fn starts_with_box(data: &[u8]) -> bool {
    let Some(header) = data.get(..HEADER_LEN) else {
        return false;
    };

    let size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
    let sized = size == 0 || size == 1 || (HEADER_LEN..=data.len()).contains(&size);

    sized && header[4..].iter().all(u8::is_ascii_graphic)
}

/// The JPEG stream inside a preview box, whose header layout differs between
/// box types but always precedes the start of image marker.
fn embedded_jpeg(payload: &[u8]) -> Option<&[u8]> {
    let start = payload
        .windows(SOI.len())
        .position(|window| window == SOI)?;

    payload.get(start..)
}

/// Iterator over the boxes of one level: `(type, payload)`.
struct Boxes<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Boxes<'a> {
    fn new(data: &'a [u8]) -> Self {
        Boxes { data, pos: 0 }
    }
}

impl<'a> Iterator for Boxes<'a> {
    type Item = ([u8; 4], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let header = self.data.get(self.pos..self.pos + HEADER_LEN)?;
        let short_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let kind = [header[4], header[5], header[6], header[7]];

        // Size 1 means a 64 bit size follows the type; size 0 means the box
        // runs to the end of the file.
        let (size, payload_at) = match short_size {
            1 => {
                let extended = self
                    .data
                    .get(self.pos + HEADER_LEN..self.pos + HEADER_LEN + 8)?;
                let size = u64::from_be_bytes(extended.try_into().ok()?) as usize;
                (size, self.pos + HEADER_LEN + 8)
            }
            0 => (self.data.len() - self.pos, self.pos + HEADER_LEN),
            _ => (short_size, self.pos + HEADER_LEN),
        };

        let end = self.pos.checked_add(size)?.min(self.data.len());
        if size < HEADER_LEN || payload_at > end {
            return None;
        }

        let payload = self.data.get(payload_at..end)?;
        self.pos = end;

        Some((kind, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = ((payload.len() + HEADER_LEN) as u32).to_be_bytes().to_vec();
        out.extend_from_slice(kind);
        out.extend_from_slice(payload);
        out
    }

    fn jpeg(size: usize) -> Vec<u8> {
        let mut out = SOI.to_vec();
        out.resize(size, 0x11);
        out
    }

    #[test]
    fn recognises_the_container() {
        let mut data = 16u32.to_be_bytes().to_vec();
        data.extend_from_slice(b"ftypcrx ");
        assert!(is_bmff(&data));
        assert!(!is_bmff(b"\xff\xd8\xff\xe1"));
    }

    #[test]
    fn finds_directories_and_the_preview() {
        let mut uuid_payload = vec![0u8; UUID_LEN];
        uuid_payload.extend_from_slice(&boxed(b"CMT1", b"II*\0root"));
        uuid_payload.extend_from_slice(&boxed(b"CMT2", b"II*\0exif"));
        uuid_payload.extend_from_slice(&boxed(b"CMT4", b"II*\0gps"));

        let mut preview_payload = vec![0u8; UUID_LEN];
        let preview = jpeg(2048);
        let mut prvw = vec![0u8; 16];
        prvw.extend_from_slice(&preview);
        preview_payload.extend_from_slice(&boxed(b"PRVW", &prvw));

        let mut data = boxed(b"ftyp", b"crx isom");
        data.extend_from_slice(&boxed(b"moov", &boxed(b"uuid", &uuid_payload)));
        data.extend_from_slice(&boxed(b"uuid", &preview_payload));

        let found = extract(&data);

        assert_eq!(found.exif.len(), 3);
        assert_eq!(found.exif[0].kind, IfdKind::Root);
        assert_eq!(found.exif[1].kind, IfdKind::Exif);
        assert_eq!(found.exif[2].kind, IfdKind::Gps);
        assert_eq!(found.preview, Some(preview.as_slice()));
    }

    #[test]
    fn prefers_the_preview_over_the_thumbnail() {
        let thumbnail = jpeg(200);
        let preview = jpeg(4000);

        let mut data = boxed(b"ftyp", b"crx isom");
        data.extend_from_slice(&boxed(b"THMB", &thumbnail));
        data.extend_from_slice(&boxed(b"PRVW", &preview));

        assert_eq!(extract(&data).preview.map(<[u8]>::len), Some(preview.len()));
    }

    #[test]
    fn finds_a_preview_behind_canons_padding() {
        let preview = jpeg(4096);
        let mut prvw = vec![0u8; 16];
        prvw.extend_from_slice(&preview);

        let mut uuid_payload = vec![0u8; UUID_LEN];
        // The eight bytes Canon writes before the first box.
        uuid_payload.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
        uuid_payload.extend_from_slice(&boxed(b"PRVW", &prvw));

        let mut data = boxed(b"ftyp", b"crx isom");
        data.extend_from_slice(&boxed(b"uuid", &uuid_payload));

        assert_eq!(extract(&data).preview, Some(preview.as_slice()));
    }

    #[test]
    fn an_extension_holding_plain_text_is_skipped() {
        let mut uuid_payload = vec![0u8; UUID_LEN];
        uuid_payload.extend_from_slice(b"<?xpacket begin='' id='W5M0MpCehiHzreSzNTczkc9d'?>");

        let mut data = boxed(b"ftyp", b"crx isom");
        data.extend_from_slice(&boxed(b"uuid", &uuid_payload));

        let found = extract(&data);
        assert!(found.exif.is_empty());
        assert!(found.preview.is_none());
    }

    #[test]
    fn a_box_with_a_bad_size_stops_the_walk() {
        let mut data = boxed(b"ftyp", b"crx isom");
        // A size smaller than the header itself.
        data.extend_from_slice(&[0, 0, 0, 2]);
        data.extend_from_slice(b"moov");

        assert!(extract(&data).exif.is_empty());
    }

    #[test]
    fn truncated_files_do_not_panic() {
        let data = boxed(b"moov", &boxed(b"CMT1", b"II*\0"));
        for len in 0..data.len() {
            let _ = extract(&data[..len]);
        }
    }
}

//! JPEG marker segment walking.

use super::{ExifBlock, Extracted};

const EXIF_SIGNATURE: &[u8] = b"Exif\0\0";
const ICC_SIGNATURE: &[u8] = b"ICC_PROFILE\0";

const MARKER_PREFIX: u8 = 0xFF;
const SOI: u8 = 0xD8;
const EOI: u8 = 0xD9;
const SOS: u8 = 0xDA;
const APP1: u8 = 0xE1;
const APP2: u8 = 0xE2;

/// True when `data` starts with a JPEG start-of-image marker.
pub fn is_jpeg(data: &[u8]) -> bool {
    data.starts_with(&[MARKER_PREFIX, SOI])
}

/// Collects the EXIF block and ICC profile from a JPEG's APP segments.
///
/// Stops at the start of scan data, which is where the interesting metadata
/// ends and megabytes of entropy coded pixels begin.
pub fn extract(data: &[u8]) -> Extracted<'_> {
    let mut out = Extracted::default();
    let mut icc_chunks: Vec<&[u8]> = Vec::new();

    for (marker, payload) in Segments::new(data) {
        match marker {
            APP1 if payload.starts_with(EXIF_SIGNATURE) && out.exif.is_empty() => {
                out.exif
                    .push(ExifBlock::root(&payload[EXIF_SIGNATURE.len()..]));
            }
            // Profiles over 64KB are split across numbered APP2 segments; the
            // two byte header is the sequence number and total count.
            APP2 if payload.starts_with(ICC_SIGNATURE) => {
                if let Some(chunk) = payload.get(ICC_SIGNATURE.len() + 2..) {
                    icc_chunks.push(chunk);
                }
            }
            _ => {}
        }
    }

    if !icc_chunks.is_empty() {
        out.icc = Some(icc_chunks.concat());
    }

    out
}

/// Iterator over `(marker, payload)` of a JPEG's segments.
struct Segments<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Segments<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            // Skip the SOI marker itself.
            pos: if is_jpeg(data) { 2 } else { data.len() },
        }
    }
}

impl<'a> Iterator for Segments<'a> {
    type Item = (u8, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Fill bytes of 0xFF are legal between segments.
            while self.data.get(self.pos) == Some(&MARKER_PREFIX)
                && self.data.get(self.pos + 1) == Some(&MARKER_PREFIX)
            {
                self.pos += 1;
            }

            if *self.data.get(self.pos)? != MARKER_PREFIX {
                return None;
            }

            let marker = *self.data.get(self.pos + 1)?;
            if marker == SOS || marker == EOI {
                return None;
            }

            // Standalone markers carry no length field.
            if matches!(marker, 0x01 | 0xD0..=0xD7) {
                self.pos += 2;
                continue;
            }

            let length =
                u16::from_be_bytes([*self.data.get(self.pos + 2)?, *self.data.get(self.pos + 3)?])
                    as usize;

            if length < 2 {
                return None;
            }

            let start = self.pos + 4;
            let payload = self.data.get(start..start.checked_add(length - 2)?)?;
            self.pos = start + length - 2;

            return Some((marker, payload));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(marker: u8, payload: &[u8]) -> Vec<u8> {
        let mut out = vec![MARKER_PREFIX, marker];
        out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(payload);
        out
    }

    fn jpeg_with(segments: &[Vec<u8>]) -> Vec<u8> {
        let mut out = vec![MARKER_PREFIX, SOI];
        for segment in segments {
            out.extend_from_slice(segment);
        }
        out.extend_from_slice(&[MARKER_PREFIX, SOS, 0, 2]);
        out.extend_from_slice(&[0u8; 32]);
        out
    }

    #[test]
    fn finds_exif_and_icc() {
        let mut exif_payload = EXIF_SIGNATURE.to_vec();
        exif_payload.extend_from_slice(b"II*\0");

        let mut icc_payload = ICC_SIGNATURE.to_vec();
        icc_payload.extend_from_slice(&[1, 1]);
        icc_payload.extend_from_slice(b"profile-bytes");

        let data = jpeg_with(&[segment(APP1, &exif_payload), segment(APP2, &icc_payload)]);
        let found = extract(&data);

        assert_eq!(found.root_exif(), Some(&b"II*\0"[..]));
        assert_eq!(found.icc.as_deref(), Some(&b"profile-bytes"[..]));
    }

    #[test]
    fn concatenates_split_icc_profiles() {
        let chunk = |seq: u8, body: &[u8]| {
            let mut payload = ICC_SIGNATURE.to_vec();
            payload.extend_from_slice(&[seq, 2]);
            payload.extend_from_slice(body);
            segment(APP2, &payload)
        };

        let data = jpeg_with(&[chunk(1, b"head"), chunk(2, b"tail")]);
        assert_eq!(extract(&data).icc.as_deref(), Some(&b"headtail"[..]));
    }

    #[test]
    fn stops_at_scan_data() {
        // An APP1-looking sequence after SOS must not be mistaken for metadata.
        let mut data = jpeg_with(&[]);
        data.extend_from_slice(&segment(APP1, &{
            let mut p = EXIF_SIGNATURE.to_vec();
            p.extend_from_slice(b"II*\0");
            p
        }));

        assert!(extract(&data).exif.is_empty());
    }

    #[test]
    fn truncated_files_do_not_panic() {
        for len in 0..24 {
            let data = jpeg_with(&[segment(APP1, EXIF_SIGNATURE)]);
            let _ = extract(&data[..len.min(data.len())]);
        }
    }

    #[test]
    fn rejects_non_jpeg() {
        assert!(!is_jpeg(b"\x89PNG"));
        assert!(extract(b"\x89PNG\r\n").exif.is_empty());
    }
}

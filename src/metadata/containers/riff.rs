//! RIFF chunk walking, used by WebP.

use super::{ExifBlock, Extracted};

const RIFF: &[u8] = b"RIFF";
const WEBP: &[u8] = b"WEBP";
const HEADER: usize = 12;
const CHUNK_HEADER: usize = 8;
const EXIF_SIGNATURE: &[u8] = b"Exif\0\0";

/// True when `data` is a RIFF container holding a WebP image.
pub fn is_webp(data: &[u8]) -> bool {
    data.starts_with(RIFF) && data.get(8..12) == Some(WEBP)
}

/// Collects the `EXIF` and `ICCP` chunks of a WebP file.
pub fn extract(data: &[u8]) -> Extracted<'_> {
    let mut out = Extracted::default();

    for (kind, payload) in Chunks::new(data) {
        match &kind {
            b"EXIF" if out.exif.is_empty() => {
                // Some encoders prefix the JPEG style signature, some don't.
                out.exif.push(ExifBlock::root(
                    payload.strip_prefix(EXIF_SIGNATURE).unwrap_or(payload),
                ));
            }
            b"ICCP" if out.icc.is_none() => out.icc = Some(payload.to_vec()),
            _ => {}
        }
    }

    out
}

/// Iterator over `(chunk fourcc, payload)`. Chunks are padded to even sizes.
struct Chunks<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Chunks<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: if is_webp(data) { HEADER } else { data.len() },
        }
    }
}

impl<'a> Iterator for Chunks<'a> {
    type Item = ([u8; 4], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let header = self.data.get(self.pos..self.pos + CHUNK_HEADER)?;
        let kind = [header[0], header[1], header[2], header[3]];
        let length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;

        let start = self.pos + CHUNK_HEADER;
        let payload = self.data.get(start..start.checked_add(length)?)?;
        self.pos = start + length + (length & 1);

        Some((kind, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = kind.to_vec();
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(payload);
        if payload.len() % 2 == 1 {
            out.push(0);
        }
        out
    }

    fn webp_with(chunks: &[Vec<u8>]) -> Vec<u8> {
        let body: Vec<u8> = chunks.concat();
        let mut out = RIFF.to_vec();
        out.extend_from_slice(&((body.len() + 4) as u32).to_le_bytes());
        out.extend_from_slice(WEBP);
        out.extend_from_slice(&body);
        out
    }

    #[test]
    fn finds_exif_and_icc_chunks() {
        let data = webp_with(&[
            chunk(b"VP8 ", &[0u8; 5]),
            chunk(b"ICCP", b"profile"),
            chunk(b"EXIF", b"II*\0body"),
        ]);

        let found = extract(&data);
        assert_eq!(found.root_exif(), Some(&b"II*\0body"[..]));
        assert_eq!(found.icc.as_deref(), Some(&b"profile"[..]));
    }

    #[test]
    fn strips_the_optional_exif_signature() {
        let mut payload = EXIF_SIGNATURE.to_vec();
        payload.extend_from_slice(b"II*\0");
        let data = webp_with(&[chunk(b"EXIF", &payload)]);

        assert_eq!(extract(&data).root_exif(), Some(&b"II*\0"[..]));
    }

    #[test]
    fn truncated_files_do_not_panic() {
        let data = webp_with(&[chunk(b"EXIF", b"II*\0")]);
        for len in 0..data.len() {
            let _ = extract(&data[..len]);
        }
    }

    #[test]
    fn rejects_non_webp_riff() {
        let mut data = RIFF.to_vec();
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(b"WAVE");
        assert!(!is_webp(&data));
        assert!(extract(&data).exif.is_empty());
    }
}

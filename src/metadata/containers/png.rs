//! PNG chunk walking.

use std::io::Read;

use flate2::read::ZlibDecoder;

use super::{ExifBlock, Extracted};

const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
/// The keyword PNG reserves for an XMP packet in an `iTXt` chunk.
const XMP_KEYWORD: &[u8] = b"XML:com.adobe.xmp";
const CHUNK_HEADER: usize = 8;
const CHUNK_CRC: usize = 4;

/// Refuse to inflate a bomb: real profiles and packets are a few hundred
/// kilobytes at most.
const MAX_INFLATED_BYTES: u64 = 16 * 1024 * 1024;

/// True when `data` carries the PNG signature.
pub fn is_png(data: &[u8]) -> bool {
    data.starts_with(SIGNATURE)
}

/// Collects the EXIF block (`eXIf`) and ICC profile (`iCCP`) from a PNG.
pub fn extract(data: &[u8]) -> Extracted<'_> {
    let mut out = Extracted::default();

    for (kind, payload) in Chunks::new(data) {
        match &kind {
            b"eXIf" if out.exif.is_empty() => out.exif.push(ExifBlock::root(payload)),
            b"iCCP" if out.icc.is_none() => out.icc = inflate_iccp(payload),
            b"iTXt" if out.xmp.is_none() => out.xmp = xmp_from_itxt(payload),
            // Everything worth reading precedes the pixel data.
            b"IDAT" => break,
            _ => {}
        }
    }

    out
}

/// An `iCCP` chunk is `name \0 compression_method zlib_data`.
fn inflate_iccp(payload: &[u8]) -> Option<Vec<u8>> {
    let name_end = payload.iter().position(|b| *b == 0)?;

    inflate(payload.get(name_end + 2..)?)
}

fn inflate(compressed: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();

    match ZlibDecoder::new(compressed)
        .take(MAX_INFLATED_BYTES)
        .read_to_end(&mut out)
    {
        Ok(_) => Some(out),
        Err(e) => {
            tracing::debug!("Failure inflating a PNG chunk -> {e}");
            None
        }
    }
}

/// An `iTXt` chunk is `keyword \0 compressed? method language \0 translated \0
/// text`, and only the one holding an XMP packet interests us.
fn xmp_from_itxt(payload: &[u8]) -> Option<Vec<u8>> {
    let keyword_end = payload.iter().position(|b| *b == 0)?;
    if &payload[..keyword_end] != XMP_KEYWORD {
        return None;
    }

    let compressed = *payload.get(keyword_end + 1)? == 1;
    // Skip the compression method, then the language and translated keyword.
    let rest = payload.get(keyword_end + 3..)?;
    let language_end = rest.iter().position(|b| *b == 0)?;
    let rest = rest.get(language_end + 1..)?;
    let translated_end = rest.iter().position(|b| *b == 0)?;
    let text = rest.get(translated_end + 1..)?;

    if compressed {
        inflate(text)
    } else {
        Some(text.to_vec())
    }
}

/// Iterator over `(chunk type, payload)`.
struct Chunks<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Chunks<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: if is_png(data) {
                SIGNATURE.len()
            } else {
                data.len()
            },
        }
    }
}

impl<'a> Iterator for Chunks<'a> {
    type Item = ([u8; 4], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let header = self.data.get(self.pos..self.pos + CHUNK_HEADER)?;
        let length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let kind = [header[4], header[5], header[6], header[7]];

        let start = self.pos + CHUNK_HEADER;
        let payload = self.data.get(start..start.checked_add(length)?)?;
        self.pos = start + length + CHUNK_CRC;

        Some((kind, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn chunk(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = (payload.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(kind);
        out.extend_from_slice(payload);
        out.extend_from_slice(&[0, 0, 0, 0]); // CRC, unchecked
        out
    }

    fn png_with(chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut out = SIGNATURE.to_vec();
        for c in chunks {
            out.extend_from_slice(c);
        }
        out.extend_from_slice(&chunk(b"IDAT", &[0u8; 8]));
        out
    }

    fn deflate(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn finds_exif_chunk() {
        let data = png_with(&[chunk(b"eXIf", b"II*\0rest")]);
        assert_eq!(extract(&data).root_exif(), Some(&b"II*\0rest"[..]));
    }

    #[test]
    fn inflates_icc_profile() {
        let mut payload = b"ICC profile\0\0".to_vec();
        payload.extend_from_slice(&deflate(b"the-profile"));

        let data = png_with(&[chunk(b"iCCP", &payload)]);
        assert_eq!(extract(&data).icc.as_deref(), Some(&b"the-profile"[..]));
    }

    #[test]
    fn finds_an_xmp_packet() {
        let mut payload = XMP_KEYWORD.to_vec();
        // Uncompressed, method 0, no language, no translated keyword.
        payload.extend_from_slice(&[0, 0, 0, 0, 0]);
        payload.extend_from_slice(b"<x:xmpmeta/>");

        let data = png_with(&[chunk(b"iTXt", &payload)]);

        assert_eq!(extract(&data).xmp.as_deref(), Some(&b"<x:xmpmeta/>"[..]));
    }

    #[test]
    fn a_text_chunk_that_is_not_xmp_is_ignored() {
        let mut payload = b"Comment".to_vec();
        payload.extend_from_slice(&[0, 0, 0, 0, 0]);
        payload.extend_from_slice(b"hello");

        let data = png_with(&[chunk(b"iTXt", &payload)]);

        assert!(extract(&data).xmp.is_none());
    }

    #[test]
    fn corrupt_icc_data_is_ignored() {
        let payload = b"name\0\0not-zlib".to_vec();
        let data = png_with(&[chunk(b"iCCP", &payload)]);
        assert!(extract(&data).icc.is_none());
    }

    #[test]
    fn truncated_files_do_not_panic() {
        let data = png_with(&[chunk(b"eXIf", b"II*\0")]);
        for len in 0..data.len() {
            let _ = extract(&data[..len]);
        }
    }

    #[test]
    fn rejects_non_png() {
        assert!(!is_png(b"\xff\xd8\xff"));
        assert!(extract(b"\xff\xd8\xff").exif.is_empty());
    }
}

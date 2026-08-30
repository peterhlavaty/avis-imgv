//! Locating metadata inside the file formats we open.
//!
//! Every container hides the same three things in a different place: a TIFF
//! block with the EXIF directories, an ICC profile, and (for raw files) an
//! embedded JPEG preview. Each sub-module knows one container; [`extract`]
//! picks the right one.

pub mod bmff;
pub mod jpeg;
pub mod png;
pub mod raw;
pub mod riff;

use crate::formats::Format;
use crate::metadata::tags::IfdKind;
use crate::metadata::tiff::Tiff;

/// A TIFF block and what its first directory means.
///
/// Most containers store one block whose IFD0 is the root directory and which
/// points at the others. Canon's CR3 instead stores each directory separately,
/// which is why the kind travels with the bytes.
#[derive(Debug)]
pub struct ExifBlock<'a> {
    pub bytes: &'a [u8],
    pub kind: IfdKind,
}

impl<'a> ExifBlock<'a> {
    /// A block whose IFD0 is the root directory.
    pub fn root(bytes: &'a [u8]) -> ExifBlock<'a> {
        ExifBlock {
            bytes,
            kind: IfdKind::Root,
        }
    }
}

/// A picture stored as plain pixels rather than as a compressed stream.
///
/// What a DNG uses for its reduced-resolution copy, and the only thing some
/// of them carry that is a picture at all.
#[derive(Debug, Clone, Copy)]
pub struct Pixels<'a> {
    pub bytes: &'a [u8],
    pub width: u32,
    pub height: u32,
    /// One for grey, three for RGB.
    pub samples: u8,
}

/// What a container walk found. Borrows from the file buffer wherever the
/// bytes are stored verbatim.
#[derive(Debug, Default)]
pub struct Extracted<'a> {
    /// TIFF blocks holding the EXIF directories.
    pub exif: Vec<ExifBlock<'a>>,
    /// ICC profile bytes. Owned because some containers store it split or
    /// compressed.
    pub icc: Option<Vec<u8>>,
    /// Embedded JPEG to decode instead of the file itself.
    pub preview: Option<&'a [u8]>,
    /// The small JPEG a camera embeds, for putting something on screen while
    /// the real one is still being decoded.
    pub thumbnail: Option<&'a [u8]>,
    /// XMP packet, holding the rating and keywords other tools wrote.
    pub xmp: Option<Vec<u8>>,
    /// An uncompressed picture to draw, when there is no compressed one.
    pub pixels: Option<Pixels<'a>>,
    /// What the photograph itself measures, which for a raw file is not what
    /// its first directory says.
    pub full_size: Option<(u32, u32)>,
}

impl<'a> Extracted<'a> {
    /// The block holding the root directory, if the walk found one.
    pub fn root_exif(&self) -> Option<&'a [u8]> {
        self.exif
            .iter()
            .find(|block| block.kind == IfdKind::Root)
            .map(|block| block.bytes)
    }
}

/// Extracts metadata locations from `data`, using `format` as a hint.
///
/// The hint comes from the file extension, so it can be wrong; the content is
/// sniffed as a fallback and wins whenever it disagrees.
pub fn extract(data: &[u8], format: Option<Format>) -> Extracted<'_> {
    match format {
        Some(Format::Jpeg) if jpeg::is_jpeg(data) => jpeg::extract(data),
        Some(Format::Png) if png::is_png(data) => png::extract(data),
        Some(Format::Webp) if riff::is_webp(data) => riff::extract(data),
        Some(Format::Raw) if bmff::is_bmff(data) => bmff::extract(data),
        Some(Format::Raw) => raw::extract(data),
        Some(Format::Tiff) => tiff_block(data),
        _ => sniff(data),
    }
}

/// Extracts using content inspection alone.
fn sniff(data: &[u8]) -> Extracted<'_> {
    if jpeg::is_jpeg(data) {
        jpeg::extract(data)
    } else if png::is_png(data) {
        png::extract(data)
    } else if riff::is_webp(data) {
        riff::extract(data)
    } else if bmff::is_bmff(data) {
        bmff::extract(data)
    } else if Tiff::new(data).is_some() {
        tiff_block(data)
    } else {
        Extracted::default()
    }
}

/// A plain TIFF is its own EXIF block; raw handling covers reading its ICC
/// profile out of tag 0x8773.
fn tiff_block(data: &[u8]) -> Extracted<'_> {
    let extracted = raw::extract(data);

    Extracted {
        exif: vec![ExifBlock::root(data)],
        icc: extracted.icc,
        xmp: extracted.xmp,
        thumbnail: extracted.thumbnail,
        // The file itself decodes, so an embedded copy is of no use.
        preview: None,
        pixels: None,
        full_size: extracted.full_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wrong_extension_falls_back_to_sniffing() {
        let mut exif_payload = b"Exif\0\0".to_vec();
        exif_payload.extend_from_slice(b"II*\0");

        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE1];
        data.extend_from_slice(&((exif_payload.len() + 2) as u16).to_be_bytes());
        data.extend_from_slice(&exif_payload);
        data.extend_from_slice(&[0xFF, 0xDA, 0, 2]);

        // Claimed to be a PNG, actually a JPEG.
        assert_eq!(
            extract(&data, Some(Format::Png)).root_exif(),
            Some(&b"II*\0"[..])
        );
    }

    #[test]
    fn unknown_data_yields_nothing() {
        let found = extract(b"just some bytes", None);
        assert!(found.exif.is_empty());
        assert!(found.icc.is_none());
        assert!(found.preview.is_none());
        assert!(found.xmp.is_none());
    }

    #[test]
    fn empty_input_does_not_panic() {
        for format in [
            None,
            Some(Format::Jpeg),
            Some(Format::Png),
            Some(Format::Webp),
            Some(Format::Raw),
            Some(Format::Tiff),
        ] {
            let _ = extract(&[], format);
        }
    }
}

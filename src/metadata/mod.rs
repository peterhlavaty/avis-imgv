//! In-process metadata reading.
//!
//! Replaces shelling out to `exiftool`: the file bytes are already in memory
//! for decoding, so the EXIF directories, the ICC profile and (for raw files)
//! the embedded preview are all read from that same buffer.

pub mod bytes;
pub mod containers;
pub mod icc;
pub mod labels;
pub mod orientation;
pub mod print;
pub mod tags;
pub mod text;
pub mod tiff;
pub mod value;
pub mod xmp;

use std::collections::BTreeMap;
use std::path::Path;

use crate::formats::Format;
use tags::IfdKind;
use tiff::{Ifd, Tiff};

pub use orientation::Orientation;
pub use text::{format_string_with_metadata, group_raw_jpg_paths};

/// Display name of the tag holding the colour profile name.
pub const PROFILE_DESCRIPTION: &str = "Profile Description";

/// Everything the viewer knows about an image file.
#[derive(Debug, Default, Clone)]
pub struct Metadata {
    /// Display ready tags, keyed by their exiftool name.
    pub tags: BTreeMap<String, String>,
    pub orientation: Orientation,
    /// Embedded ICC profile, when the file carries one.
    pub icc: Option<Vec<u8>>,
    /// Star rating and keywords the file itself carries, which a sidecar may
    /// then override.
    pub xmp: xmp::Xmp,
}

impl Metadata {
    /// Reads the metadata of a file already loaded into `data`.
    ///
    /// `format` is the extension based guess; the container walk falls back to
    /// sniffing when it does not match the bytes.
    pub fn parse(data: &[u8], format: Option<Format>) -> (Metadata, Option<&[u8]>) {
        let mut found = containers::extract(data, format);

        // Fuji raws keep their EXIF inside the embedded preview's own APP1.
        if found.exif.is_empty() {
            if let Some(preview) = found.preview {
                found.exif = containers::jpeg::extract(preview).exif;
            }
        }

        let mut metadata = Metadata::default();
        let mut embedded_icc = found.icc;
        let mut embedded_xmp = found.xmp;

        for block in &found.exif {
            let Some(tiff) = Tiff::new(block.bytes) else {
                continue;
            };

            metadata.read_block(&tiff, block.kind);
            embedded_icc = embedded_icc.or_else(|| bytes_of_tag(&tiff, tags::ICC_PROFILE));
            embedded_xmp = embedded_xmp.or_else(|| bytes_of_tag(&tiff, tags::XMP_PACKET));
        }

        metadata.add_composite_tags();
        metadata.icc = embedded_icc;
        metadata.read_annotations(embedded_xmp.as_deref());
        metadata.resolve_profile_description();

        (metadata, found.preview)
    }

    /// Adds the tags derived from the file itself rather than its contents.
    pub fn add_file_tags(&mut self, path: &Path, byte_len: usize) {
        if let Some(name) = path.file_name() {
            self.insert("File Name", name.to_string_lossy());
        }
        if let Some(parent) = path.parent() {
            self.insert("Directory", parent.to_string_lossy());
        }

        self.insert("File Size", format_byte_size(byte_len));
    }

    /// Records the dimensions of the decoded image.
    pub fn add_size_tags(&mut self, width: u32, height: u32) {
        self.insert("Image Size", format!("{width}x{height}"));
        self.insert(
            "Megapixels",
            value::format_f64(((width as f64 * height as f64) / 1_000_000.0 * 10.0).round() / 10.0),
        );
    }

    /// Reads the rating and keywords the file carries.
    ///
    /// XMP is authoritative; the EXIF rating tag is what Windows Explorer
    /// writes and is only consulted when there is no packet.
    fn read_annotations(&mut self, packet: Option<&[u8]>) {
        let parsed = packet
            .map(String::from_utf8_lossy)
            .and_then(|document| xmp::read(&document));

        if let Some(parsed) = parsed {
            self.xmp = parsed;
        }

        if self.xmp.rating == 0 {
            if let Some(rating) = self.tags.get("Rating").and_then(|r| xmp::parse_rating(r)) {
                self.xmp.rating = rating;
            }
        }
    }

    /// The colour profile name, used to pick an input profile for conversion.
    pub fn profile_description(&self) -> Option<&str> {
        self.tags.get(PROFILE_DESCRIPTION).map(String::as_str)
    }

    fn insert(&mut self, key: &str, value: impl Into<String>) {
        self.tags.insert(key.to_string(), value.into());
    }

    /// Reads one TIFF block whose first directory is of `kind`.
    ///
    /// A root directory also points at the EXIF and GPS ones; the other kinds
    /// stand alone, which is how Canon's CR3 stores them.
    fn read_block(&mut self, tiff: &Tiff, kind: IfdKind) {
        let Some(first) = tiff.ifds().into_iter().next() else {
            return;
        };

        if let Some(raw) = first.u32(tags::ORIENTATION) {
            self.orientation = Orientation::from_exif(raw);
        }

        self.read_ifd(&first, kind);

        if kind != IfdKind::Root {
            return;
        }

        for (pointer, kind) in [
            (tags::EXIF_IFD_POINTER, IfdKind::Exif),
            (tags::GPS_IFD_POINTER, IfdKind::Gps),
        ] {
            let Some(ifd) = first.u32(pointer).and_then(|o| tiff.ifd_at(o)) else {
                continue;
            };

            self.read_ifd(&ifd, kind);

            if kind == IfdKind::Exif {
                if let Some(interop) = ifd
                    .u32(tags::INTEROP_IFD_POINTER)
                    .and_then(|o| tiff.ifd_at(o))
                {
                    self.read_ifd(&interop, IfdKind::Interop);
                }
            }
        }
    }

    fn read_ifd(&mut self, ifd: &Ifd, kind: IfdKind) {
        for (tag, entry) in &ifd.entries {
            if tags::is_hidden(*tag) {
                continue;
            }

            let Some(name) = tags::name(kind, *tag) else {
                continue;
            };

            let Some(value) = entry.value() else {
                continue;
            };

            self.tags
                .insert(name.to_string(), print::display(kind, *tag, value));
        }
    }

    /// Tags exiftool computes rather than reads, and which the default name
    /// format relies on.
    fn add_composite_tags(&mut self) {
        if let Some(aperture) = self
            .tags
            .get("F Number")
            .or_else(|| self.tags.get("Aperture Value"))
            .cloned()
        {
            self.insert("Aperture", aperture);
        }

        if let Some(shutter) = self
            .tags
            .get("Exposure Time")
            .or_else(|| self.tags.get("Shutter Speed Value"))
            .cloned()
        {
            self.insert("Shutter Speed", shutter);
        }
    }

    /// Names the colour profile, preferring the embedded one over the hint in
    /// the `Color Space` tag.
    fn resolve_profile_description(&mut self) {
        let described = self.icc.as_deref().and_then(icc::description);

        let description = described.or_else(|| match self.tags.get("Color Space") {
            Some(space) if space != "Uncalibrated" => Some(space.clone()),
            _ => None,
        });

        if let Some(description) = description {
            self.insert(PROFILE_DESCRIPTION, description);
        }
    }
}

/// TIFF and DNG keep blobs such as the ICC profile and the XMP packet in tags
/// rather than in container chunks.
fn bytes_of_tag(tiff: &Tiff<'_>, tag: u16) -> Option<Vec<u8>> {
    tiff.ifds()
        .iter()
        .find_map(|ifd| ifd.entry(tag))
        .and_then(|entry| tiff.entry_bytes(entry))
        .map(<[u8]>::to_vec)
}

fn format_byte_size(bytes: usize) -> String {
    const UNITS: &[(f64, &str)] = &[(1e9, "GB"), (1e6, "MB"), (1e3, "kB")];
    let bytes = bytes as f64;

    for (scale, unit) in UNITS {
        if bytes >= *scale {
            return format!(
                "{} {unit}",
                value::format_f64((bytes / scale * 10.0).round() / 10.0)
            );
        }
    }

    format!("{} bytes", bytes as u64)
}

#[cfg(test)]
mod tests {
    use super::tiff::test_support::{build_tiff, build_tiff_with_sub_ifd, rational};
    use super::value::FieldType;
    use super::*;

    /// Wraps a TIFF block in the JPEG APP1 segment a camera would write.
    fn jpeg_with_exif(tiff_block: &[u8]) -> Vec<u8> {
        let mut payload = b"Exif\0\0".to_vec();
        payload.extend_from_slice(tiff_block);

        let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
        out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(&payload);
        out.extend_from_slice(&[0xFF, 0xDA, 0, 2]);
        out
    }

    #[test]
    fn reads_tags_out_of_a_jpeg() {
        let block = build_tiff(&[
            (0x0110, FieldType::Ascii, 8, b"X-T5\0\0\0\0".to_vec()),
            (
                tags::ORIENTATION,
                FieldType::Short,
                1,
                6u16.to_le_bytes().to_vec(),
            ),
        ]);

        let jpeg = jpeg_with_exif(&block);
        let (metadata, preview) = Metadata::parse(&jpeg, Some(Format::Jpeg));

        assert_eq!(
            metadata.tags.get("Camera Model Name").map(String::as_str),
            Some("X-T5")
        );
        assert_eq!(metadata.orientation, Orientation::Rotate90Cw);
        assert!(preview.is_none());
    }

    #[test]
    fn derives_the_composite_tags_the_name_format_uses() {
        let block = build_tiff_with_sub_ifd(
            &[(0x0110, FieldType::Ascii, 8, b"X-T5\0\0\0\0".to_vec())],
            tags::EXIF_IFD_POINTER,
            &[
                (0x829A, FieldType::Rational, 1, rational(1, 500)),
                (0x829D, FieldType::Rational, 1, rational(56, 10)),
                (0x8827, FieldType::Short, 1, 400u16.to_le_bytes().to_vec()),
            ],
        );

        let jpeg = jpeg_with_exif(&block);
        let (metadata, _) = Metadata::parse(&jpeg, Some(Format::Jpeg));

        assert_eq!(metadata.tags.get("ISO").map(String::as_str), Some("400"));
        assert_eq!(
            metadata.tags.get("Shutter Speed").map(String::as_str),
            Some("1/500")
        );
        assert_eq!(
            metadata.tags.get("Aperture").map(String::as_str),
            Some("5.6")
        );
    }

    #[test]
    fn reads_a_rating_and_keywords_from_an_embedded_packet() {
        let packet = crate::metadata::xmp::update(
            None,
            &xmp::Xmp {
                rating: 4,
                keywords: vec!["Keeper".to_string()],
            },
        );

        let mut payload = b"http://ns.adobe.com/xap/1.0/ ".to_vec();
        payload.extend_from_slice(packet.as_bytes());

        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1];
        jpeg.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        jpeg.extend_from_slice(&payload);
        jpeg.extend_from_slice(&[0xFF, 0xDA, 0, 2]);

        let (metadata, _) = Metadata::parse(&jpeg, Some(Format::Jpeg));

        assert_eq!(metadata.xmp.rating, 4);
        assert_eq!(metadata.xmp.keywords, vec!["Keeper"]);
    }

    #[test]
    fn the_exif_rating_tag_stands_in_for_a_missing_packet() {
        let block = build_tiff(&[(
            tags::RATING,
            FieldType::Short,
            1,
            3u16.to_le_bytes().to_vec(),
        )]);

        let jpeg = jpeg_with_exif(&block);
        let (metadata, _) = Metadata::parse(&jpeg, Some(Format::Jpeg));

        assert_eq!(metadata.xmp.rating, 3);
    }

    #[test]
    fn file_and_size_tags_are_added() {
        let mut metadata = Metadata::default();
        metadata.add_file_tags(Path::new("/photos/trip/DSCF0001.JPG"), 5_400_000);
        metadata.add_size_tags(6000, 4000);

        assert_eq!(
            metadata.tags.get("File Name").map(String::as_str),
            Some("DSCF0001.JPG")
        );
        assert!(metadata
            .tags
            .get("Directory")
            .is_some_and(|d| d.ends_with("trip")));
        assert_eq!(
            metadata.tags.get("File Size").map(String::as_str),
            Some("5.4 MB")
        );
        assert_eq!(
            metadata.tags.get("Image Size").map(String::as_str),
            Some("6000x4000")
        );
        assert_eq!(
            metadata.tags.get("Megapixels").map(String::as_str),
            Some("24")
        );
    }

    #[test]
    fn colour_space_names_the_profile_when_none_is_embedded() {
        let mut metadata = Metadata {
            tags: BTreeMap::from([("Color Space".to_string(), "sRGB".to_string())]),
            ..Default::default()
        };
        metadata.resolve_profile_description();

        assert_eq!(metadata.profile_description(), Some("sRGB"));
    }

    #[test]
    fn files_without_metadata_parse_to_defaults() {
        let (metadata, preview) = Metadata::parse(b"not an image", None);
        assert!(metadata.tags.is_empty());
        assert_eq!(metadata.orientation, Orientation::Normal);
        assert!(preview.is_none());
    }
}

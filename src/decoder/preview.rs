//! Getting something on screen before the real image exists.
//!
//! Decoding a 24 megapixel JPEG takes about 145ms. The camera that took it
//! also stored a thumbnail of a few kilobytes in the same file, near the
//! front, and that decodes in under a millisecond. Reading the first part of
//! the file gives both the metadata and something to show, so a window is
//! never empty and the side panel is never blank while the decoders work.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use image::RgbaImage;

use crate::formats::Format;
use crate::metadata::{containers, Metadata, Orientation};

/// How much of the file to read.
///
/// EXIF lives near the front and a thumbnail lives inside it, capped at 64KB
/// by the format. Half a megabyte covers that with room for the raw files that
/// put their directories a little further in.
const HEAD_BYTES: usize = 512 * 1024;

/// What a file says about itself before anything is decoded properly.
pub struct Preview {
    pub metadata: Metadata,
    /// The camera's thumbnail, decoded. Absent for files that have none.
    pub image: Option<RgbaImage>,
    /// Size of the real image, so the thumbnail can stand in for it at the
    /// right size and nothing moves when the real one arrives.
    pub full_size: (u32, u32),
    pub orientation: Orientation,
}

impl Preview {
    /// Whether there is anything to put on screen, as opposed to only
    /// metadata.
    pub fn has_image(&self) -> bool {
        self.image.is_some()
    }
}

/// Reads the front of `path` and returns what it can make of it.
///
/// `output_profile` is the display's, and the camera's thumbnail is converted
/// into it exactly as the photograph itself is. Without that the thumbnail was
/// a different colour from the photograph it stands in for: a camera set to
/// Adobe RGB writes its preview in Adobe RGB, so the contact sheet was drawn
/// flat and undersaturated, and every image visibly shifted the instant the
/// real decode landed underneath it.
///
/// Returns `None` only when the file cannot be read at all; a file with no
/// metadata and no thumbnail still yields an empty preview, which is worth
/// having because it records that the file was looked at.
pub fn load(path: &Path, output_profile: &str) -> Option<Preview> {
    let head = read_head(path)?;
    let format = Format::from_path(path);

    let parsed = Metadata::parse(&head, format);
    let mut metadata = parsed.metadata;

    let orientation = if format.is_some_and(Format::ignores_exif_orientation) {
        Orientation::Normal
    } else {
        metadata.orientation
    };

    // And the turn the user asked for, so a thumbnail agrees with the
    // photograph it stands for. Read on this thread, which is a worker.
    let orientation = orientation.then(
        crate::annotations::sidecar::read(path)
            .map(|xmp| xmp.orientation)
            .unwrap_or_default(),
    );

    let mut image = parsed.thumbnail.and_then(decode_thumbnail);

    // The thumbnail carries no profile of its own; what describes it is the
    // file's, which is what the photograph will be converted from too.
    if let Some(thumbnail) = image.as_mut() {
        super::color::convert(thumbnail, &metadata, output_profile);
    }
    // What the container worked out beats what the tags say: a raw file's
    // first directory describes a reduced-resolution copy of the photograph
    // rather than the photograph.
    let full_size = parsed
        .full_size
        .or_else(|| full_size(&head, format, &metadata))
        .unwrap_or_else(|| {
            image
                .as_ref()
                .map(|image| (image.width(), image.height()))
                .unwrap_or((0, 0))
        });

    // The file's own length, not the length of the part of it that was read.
    // Half a megabyte is all that is ever read here, so every raw file in the
    // side panel used to claim to be 512 kB.
    metadata.add_file_tags(path, byte_len(path));

    Some(Preview {
        metadata,
        image,
        full_size,
        orientation,
    })
}

/// The front of a file, for a caller that wants to read more than one thing
/// out of it.
///
/// What the folder modes want: sorting, filtering and listing the timestamps
/// of a thousand files needs what every one of them says, and nothing at all
/// of what they look like.
pub fn head(path: &Path) -> Option<Vec<u8>> {
    read_head(path)
}

/// Reads at most [`HEAD_BYTES`] from the front of the file.
/// How big the file actually is, or nought when it cannot be asked.
///
/// Nought rather than a guess: a size that is missing is obvious, and a size
/// that is wrong is believed.
fn byte_len(path: &Path) -> usize {
    std::fs::metadata(path)
        .map(|file| file.len() as usize)
        .unwrap_or(0)
}

fn read_head(path: &Path) -> Option<Vec<u8>> {
    let file = File::open(path)
        .map_err(|e| tracing::debug!("{} could not be opened: {e}", path.display()))
        .ok()?;

    let mut head = Vec::with_capacity(HEAD_BYTES.min(1 << 16));
    file.take(HEAD_BYTES as u64).read_to_end(&mut head).ok()?;

    Some(head)
}

fn decode_thumbnail(bytes: &[u8]) -> Option<RgbaImage> {
    super::codec::decode(bytes, Some(Format::Jpeg)).ok()
}

/// The dimensions of the real image.
///
/// The frame header of a JPEG is the reliable answer and sits before the
/// pixels; for everything else the EXIF tags are what there is.
fn full_size(head: &[u8], format: Option<Format>, metadata: &Metadata) -> Option<(u32, u32)> {
    if format != Some(Format::Raw) {
        if let Some(dimensions) = containers::jpeg::dimensions(head) {
            return Some(dimensions);
        }
    }

    let tag = |name: &str| metadata.tags.get(name)?.parse::<u32>().ok();

    let width = tag("Exif Image Width").or_else(|| tag("Image Width"))?;
    let height = tag("Exif Image Height").or_else(|| tag("Image Height"))?;

    (width > 0 && height > 0).then_some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::test_support::encode;
    use image::ImageFormat;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-preview-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn a_plain_jpeg_reports_its_size_without_being_decoded() {
        let dir = temp_dir("size");
        let path = dir.join("photo.jpg");
        std::fs::write(
            &path,
            encode(640, 480, [10, 20, 30, 255], ImageFormat::Jpeg),
        )
        .unwrap();

        let preview = load(&path, "srgb").expect("reads");

        assert_eq!(preview.full_size, (640, 480));
        // No camera wrote it, so there is no thumbnail inside.
        assert!(!preview.has_image());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_file_tags_are_filled_in() {
        let dir = temp_dir("tags");
        let path = dir.join("photo.jpg");
        std::fs::write(&path, encode(32, 32, [0, 0, 0, 255], ImageFormat::Jpeg)).unwrap();

        let preview = load(&path, "srgb").unwrap();

        assert_eq!(
            preview.metadata.tags.get("File Name").map(String::as_str),
            Some("photo.jpg")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_that_is_not_an_image_still_reads() {
        let dir = temp_dir("garbage");
        let path = dir.join("notes.jpg");
        std::fs::write(&path, b"this is not an image").unwrap();

        let preview = load(&path, "srgb").expect("the file exists, so it is read");

        assert!(!preview.has_image());
        assert_eq!(preview.full_size, (0, 0));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The thumbnail is converted into the display's profile, exactly as the
    /// photograph it stands in for is.
    ///
    /// Asserted by loading the same file twice: once asking for the profile
    /// the file is already in, which converts nothing, and once asking for
    /// sRGB, which has to. Identical pixels would mean the preview tier had
    /// been left unmanaged — which it was, so a camera set to Adobe RGB drew a
    /// flat contact sheet and every image shifted colour the moment the real
    /// decode landed under it.
    #[test]
    fn a_camera_thumbnail_is_converted_into_the_displays_profile() {
        let dir = temp_dir("managed");
        let path = dir.join("wide.jpg");
        std::fs::write(&path, wide_gamut_jpeg()).unwrap();

        let unconverted = load(&path, "ClayRGB").expect("reads");
        let converted = load(&path, "srgb").expect("reads");

        let (before, after) = (
            unconverted.image.expect("a thumbnail"),
            converted.image.expect("a thumbnail"),
        );

        assert_ne!(
            before.as_raw(),
            after.as_raw(),
            "the thumbnail was not colour managed"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A JPEG with a wide gamut ICC profile and a thumbnail inside it.
    fn wide_gamut_jpeg() -> Vec<u8> {
        let profile = crate::metadata::icc::built_in("Adobe RGB (1998)")
            .expect("the wide gamut profile is shipped");

        // APP2, `ICC_PROFILE `, chunk one of one, then the profile itself.
        let mut payload = b"ICC_PROFILE ".to_vec();
        payload.push(1);
        payload.push(1);
        payload.extend_from_slice(profile);

        let mut segment = vec![0xFF, 0xE2];
        segment.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        segment.extend_from_slice(&payload);

        // After the start of image marker, before everything else.
        let mut out = jpeg_with_thumbnail((400, 300), (60, 45));
        out.splice(2..2, segment);

        out
    }

    /// A JPEG carrying an EXIF thumbnail, the way a camera writes one.
    fn jpeg_with_thumbnail(full: (u32, u32), thumb: (u32, u32)) -> Vec<u8> {
        use crate::metadata::tags;
        use crate::metadata::tiff::test_support::build_tiff;
        use crate::metadata::value::FieldType;

        let thumbnail = encode(thumb.0, thumb.1, [200, 100, 50, 255], ImageFormat::Jpeg);

        // The offset is only known once the directory is laid out, so the
        // block is built twice: the first pass gives its length.
        let entry = |offset: u32| {
            vec![
                (
                    tags::JPEG_INTERCHANGE_FORMAT,
                    FieldType::Long,
                    1,
                    offset.to_le_bytes().to_vec(),
                ),
                (
                    tags::JPEG_INTERCHANGE_FORMAT_LENGTH,
                    FieldType::Long,
                    1,
                    (thumbnail.len() as u32).to_le_bytes().to_vec(),
                ),
            ]
        };

        let at = build_tiff(&entry(0)).len() as u32;
        let mut exif = build_tiff(&entry(at));
        exif.extend_from_slice(&thumbnail);

        let mut payload = b"Exif\0\0".to_vec();
        payload.extend_from_slice(&exif);

        let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
        out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(&payload);

        // The frame header of the image the thumbnail stands for.
        let image = encode(full.0, full.1, [10, 20, 30, 255], ImageFormat::Jpeg);
        out.extend_from_slice(&image[2..]);

        out
    }

    #[test]
    fn a_camera_thumbnail_is_decoded_and_reports_the_full_size() {
        let dir = temp_dir("thumbnail");
        let path = dir.join("photo.jpg");
        std::fs::write(&path, jpeg_with_thumbnail((4000, 3000), (160, 120))).unwrap();

        let preview = load(&path, "srgb").expect("reads");
        let image = preview.image.expect("the thumbnail was decoded");

        assert_eq!((image.width(), image.height()), (160, 120));
        // What is drawn is the thumbnail; what it claims to be is the image.
        assert_eq!(preview.full_size, (4000, 3000));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_file_yields_nothing() {
        assert!(load(Path::new("does-not-exist.jpg"), "srgb").is_none());
    }

    #[test]
    fn only_the_front_of_a_file_is_read() {
        let dir = temp_dir("head");
        let path = dir.join("big.jpg");

        let mut bytes = encode(64, 64, [1, 2, 3, 255], ImageFormat::Jpeg);
        bytes.resize(HEAD_BYTES * 4, 0);
        std::fs::write(&path, &bytes).unwrap();

        // The frame header is at the front, so the size is known regardless.
        assert_eq!(load(&path, "srgb").unwrap().full_size, (64, 64));

        let _ = std::fs::remove_dir_all(&dir);
    }
}

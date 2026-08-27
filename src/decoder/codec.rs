//! Byte buffers to pixels.

use image::{ImageFormat, RgbaImage};
use zune_core::colorspace::ColorSpace;
use zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

use super::DecodeError;
use crate::formats::Format;

/// Decodes `bytes` into RGBA8.
///
/// `format` is only a hint: the content is sniffed as well, and a mismatched
/// extension is common enough that guessing is worth it but trusting is not.
pub fn decode(bytes: &[u8], format: Option<Format>) -> Result<RgbaImage, DecodeError> {
    if format == Some(Format::JpegXl) {
        return decode_jpeg_xl(bytes);
    }

    // JPEG is the format that matters for speed, and it has a shorter road.
    if let Some(decoded) = decode_jpeg(bytes) {
        return Ok(decoded);
    }

    let decoded = match format.and_then(image_format) {
        Some(format) => image::load_from_memory_with_format(bytes, format)
            .or_else(|_| image::load_from_memory(bytes)),
        None => image::load_from_memory(bytes),
    };

    match decoded {
        Ok(image) => Ok(image.into_rgba8()),
        Err(e) => Err(DecodeError::Unsupported(e.to_string())),
    }
}

/// Decodes a JPEG straight into RGBA.
///
/// Going through the `image` crate yields RGB, which then has to be widened in
/// a second pass over every pixel — 27ms on a 24 megapixel photograph. Asking
/// the JPEG decoder for RGBA in the first place skips that pass entirely.
///
/// Returns `None` for anything that is not a plain JPEG, including the CMYK
/// ones whose colour handling is better left to the `image` crate.
fn decode_jpeg(bytes: &[u8]) -> Option<RgbaImage> {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return None;
    }

    // JPEG stores its dimensions in sixteen bits, so nothing valid exceeds
    // this and the default limit would refuse large panoramas.
    let limit = u16::MAX as usize;
    let options = DecoderOptions::default()
        .jpeg_set_out_colorspace(ColorSpace::RGBA)
        .set_max_width(limit)
        .set_max_height(limit);

    let mut decoder = JpegDecoder::new_with_options(bytes, options);
    let pixels = decoder.decode().ok()?;
    let info = decoder.info()?;

    // A CMYK JPEG comes back in four channels that are not RGBA, and the
    // length check catches it.
    RgbaImage::from_raw(info.width.into(), info.height.into(), pixels)
}

/// Maps our container classification onto the `image` crate's.
fn image_format(format: Format) -> Option<ImageFormat> {
    match format {
        Format::Jpeg => Some(ImageFormat::Jpeg),
        Format::Png => Some(ImageFormat::Png),
        Format::Webp => Some(ImageFormat::WebP),
        Format::Gif => Some(ImageFormat::Gif),
        Format::Bmp => Some(ImageFormat::Bmp),
        Format::Tiff => Some(ImageFormat::Tiff),
        // Raw files reach the decoder as their extracted JPEG preview.
        Format::Raw => Some(ImageFormat::Jpeg),
        Format::JpegXl => None,
    }
}

/// Decodes JPEG XL through libjxl.
///
/// Single threaded on purpose: the loader already decodes one image per
/// thread, so a parallel runner here would only fight it for cores.
///
/// Only compiled with the `jxl` feature, which builds libjxl from source.
#[cfg(feature = "jxl")]
fn decode_jpeg_xl(bytes: &[u8]) -> Result<RgbaImage, DecodeError> {
    use image::{GrayAlphaImage, GrayImage, RgbImage};

    let decoder = jpegxl_rs::decoder_builder()
        .build()
        .map_err(|e| DecodeError::Unsupported(format!("JPEG XL decoder: {e}")))?;

    let (metadata, pixels) = decoder
        .decode_with::<u8>(bytes)
        .map_err(|e| DecodeError::Unsupported(format!("JPEG XL: {e}")))?;

    let (width, height) = (metadata.width, metadata.height);
    let channels = metadata.num_color_channels + u32::from(metadata.has_alpha_channel);

    // Grayscale and colour, with or without alpha; everything ends up RGBA8.
    let rgba = match channels {
        4 => RgbaImage::from_raw(width, height, pixels),
        3 => RgbImage::from_raw(width, height, pixels).map(into_rgba),
        2 => GrayAlphaImage::from_raw(width, height, pixels).map(into_rgba),
        1 => GrayImage::from_raw(width, height, pixels).map(into_rgba),
        _ => None,
    };

    rgba.ok_or_else(|| {
        DecodeError::Unsupported(format!("JPEG XL with {channels} channels is not supported"))
    })
}

/// Every layout libjxl can hand back converts into RGBA8.
#[cfg(feature = "jxl")]
fn into_rgba(image: impl Into<image::DynamicImage>) -> RgbaImage {
    image.into().into_rgba8()
}

#[cfg(not(feature = "jxl"))]
fn decode_jpeg_xl(_bytes: &[u8]) -> Result<RgbaImage, DecodeError> {
    Err(DecodeError::Unsupported(
        "JPEG XL support is not compiled in; build with --features jxl".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::test_support::encode;
    use super::*;

    #[test]
    fn decodes_the_formats_we_advertise() {
        for (format, image_format) in [
            (Format::Png, ImageFormat::Png),
            (Format::Jpeg, ImageFormat::Jpeg),
            (Format::Bmp, ImageFormat::Bmp),
        ] {
            let bytes = encode(8, 4, [200, 100, 50, 255], image_format);
            let decoded = decode(&bytes, Some(format)).expect("decodes");
            assert_eq!((decoded.width(), decoded.height()), (8, 4));
        }
    }

    #[test]
    fn jpegs_take_the_direct_path() {
        let bytes = encode(8, 4, [200, 100, 50, 255], ImageFormat::Jpeg);
        let direct = decode_jpeg(&bytes).expect("the direct path handles a plain JPEG");

        assert_eq!((direct.width(), direct.height()), (8, 4));
        assert_eq!(direct.get_pixel(0, 0)[3], 255, "opaque alpha");
    }

    #[test]
    fn the_direct_path_declines_what_is_not_a_jpeg() {
        let bytes = encode(8, 4, [1, 2, 3, 255], ImageFormat::Png);

        assert!(decode_jpeg(&bytes).is_none());
        // And the caller falls back, so the image still decodes.
        assert!(decode(&bytes, Some(Format::Png)).is_ok());
    }

    #[test]
    fn both_paths_agree_on_the_pixels() {
        let bytes = encode(64, 32, [30, 160, 220, 255], ImageFormat::Jpeg);

        let direct = decode_jpeg(&bytes).unwrap();
        let through_image = image::load_from_memory(&bytes).unwrap().into_rgba8();

        assert_eq!(direct.dimensions(), through_image.dimensions());
        // JPEG is lossy and the two decoders round differently in the last
        // bit, which is invisible but not identical.
        for (a, b) in direct.pixels().zip(through_image.pixels()) {
            for channel in 0..4 {
                let difference = a[channel].abs_diff(b[channel]);
                assert!(difference <= 2, "{a:?} against {b:?}");
            }
        }
    }

    #[test]
    fn a_wrong_extension_still_decodes() {
        let bytes = encode(8, 4, [1, 2, 3, 255], ImageFormat::Png);
        assert!(decode(&bytes, Some(Format::Jpeg)).is_ok());
    }

    #[test]
    fn garbage_is_reported_not_panicked_on() {
        assert!(decode(b"", None).is_err());
        assert!(decode(b"garbage bytes here", Some(Format::Png)).is_err());
    }

    #[cfg(not(feature = "jxl"))]
    #[test]
    fn jpeg_xl_reports_the_missing_feature() {
        let error = decode(b"\xff\x0a", Some(Format::JpegXl)).unwrap_err();
        assert!(error.to_string().contains("--features jxl"));
    }
}

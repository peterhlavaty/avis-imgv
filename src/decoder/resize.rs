//! Downscaling, using SIMD accelerated resampling.

use fast_image_resize::images::{Image as FirImage, ImageRef};
use fast_image_resize::{PixelType, ResizeOptions, Resizer};
use image::RgbaImage;

use crate::formats::Format;

/// Whether the pixels being reduced might not be opaque.
///
/// Asked of the format rather than of the pixels wherever the format can
/// answer it, because looking is a pass over every pixel and was measured at
/// five per cent of the viewer's throughput on a folder of photographs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alpha {
    /// Nothing in this file can be transparent — a JPEG, a raw.
    Opaque,
    /// It might be; look before resampling.
    Perhaps,
}

impl Alpha {
    /// What a file of this format can hold. An unrecognised one is assumed to
    /// be able to hold anything.
    pub fn of(format: Option<Format>) -> Alpha {
        match format {
            Some(format) if !format.may_have_alpha() => Alpha::Opaque,
            _ => Alpha::Perhaps,
        }
    }
}

/// Shrinks `image` so its longest edge is at most `max_edge`.
///
/// Images already within the limit are returned untouched, and upscaling is
/// never done here: enlarging is the GPU's job at draw time.
pub fn to_max_edge(image: RgbaImage, max_edge: Option<u32>, alpha: Alpha) -> RgbaImage {
    match max_edge.and_then(|edge| reduced(&image, edge, alpha)) {
        Some(smaller) => smaller,
        // Either there was no limit, the image already fits, or the resizer
        // refused; in every case the original is the right answer.
        None => image,
    }
}

/// A smaller copy of `image`, or `None` if it already fits within `max_edge`.
///
/// Borrows rather than consuming, because the caller keeps the original: a
/// reduced copy is what goes to the GPU while the full resolution stays in
/// RAM for zooming.
pub fn reduced(image: &RgbaImage, max_edge: u32, alpha: Alpha) -> Option<RgbaImage> {
    let (width, height) = target_size(image.width(), image.height(), max_edge)?;

    // `RgbaImage` guarantees the buffer matches its dimensions, so wrapping it
    // cannot fail; the branch exists only to avoid an unwrap.
    let source = ImageRef::new(
        image.width(),
        image.height(),
        image.as_raw(),
        PixelType::U8x4,
    )
    .map_err(|e| tracing::error!("Failure wrapping image for resize -> {e}"))
    .ok()?;

    let mut destination = FirImage::new(width, height, PixelType::U8x4);

    // Resampling has to happen on premultiplied values wherever the alpha is
    // not uniform, or the colour under a transparent pixel — which is
    // arbitrary, and in a lot of PNGs is black — bleeds into its neighbours
    // and leaves a dark halo around everything with a soft edge.
    //
    // This used to pass `false` unconditionally, on the grounds that "our
    // alpha is opaque for every format that matters" — true of JPEG and raw,
    // untrue of PNG, WebP, GIF and TIFF. Whether it is opaque *here* is
    // answered by the format wherever the format can answer it, and only
    // looked at otherwise: looking is a pass over every pixel, and measured at
    // five per cent of the viewer's throughput on a folder of photographs.
    //
    // The filter is left at Lanczos3. Reducing a 24 megapixel photograph is
    // bound by reading it, not by the filter: measured against CatmullRom,
    // Bilinear and super sampling, none moved throughput, so the sharpest one
    // wins by default.
    let premultiply = alpha == Alpha::Perhaps && has_transparency(image);
    let options = ResizeOptions::new().use_alpha(premultiply);

    Resizer::new()
        .resize(&source, &mut destination, &options)
        .map_err(|e| tracing::error!("Failure resizing image -> {e}"))
        .ok()?;

    RgbaImage::from_raw(width, height, destination.into_vec())
}

/// Whether any pixel is less than fully opaque.
///
/// Chunked over the raw buffer rather than over the pixel iterator, because
/// this runs on every reduction and the whole point is that it costs nothing
/// worth measuring on the photographs that do not need it.
fn has_transparency(image: &RgbaImage) -> bool {
    image
        .as_raw()
        .as_chunks::<4>()
        .0
        .iter()
        .any(|pixel| pixel[3] != u8::MAX)
}

/// Dimensions that fit `max_edge` while preserving the aspect ratio, or `None`
/// when the image already fits.
fn target_size(width: u32, height: u32, max_edge: u32) -> Option<(u32, u32)> {
    let longest = width.max(height);
    if longest <= max_edge || max_edge == 0 {
        return None;
    }

    let scale = max_edge as f64 / longest as f64;
    let target = (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    );

    Some(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn image(width: u32, height: u32) -> RgbaImage {
        RgbaImage::from_pixel(width, height, Rgba([12, 34, 56, 255]))
    }

    #[test]
    fn keeps_the_aspect_ratio() {
        let resized = to_max_edge(image(400, 200), Some(100), Alpha::Opaque);
        assert_eq!((resized.width(), resized.height()), (100, 50));

        let resized = to_max_edge(image(200, 400), Some(100), Alpha::Opaque);
        assert_eq!((resized.width(), resized.height()), (50, 100));
    }

    #[test]
    fn never_upscales() {
        let resized = to_max_edge(image(40, 20), Some(100), Alpha::Opaque);
        assert_eq!((resized.width(), resized.height()), (40, 20));
    }

    #[test]
    fn no_limit_is_a_no_op() {
        let resized = to_max_edge(image(40, 20), None, Alpha::Opaque);
        assert_eq!((resized.width(), resized.height()), (40, 20));
    }

    #[test]
    fn extreme_ratios_keep_at_least_one_pixel() {
        let resized = to_max_edge(image(1000, 1), Some(10), Alpha::Opaque);
        assert_eq!((resized.width(), resized.height()), (10, 1));
    }

    #[test]
    fn a_reduced_copy_leaves_the_original_alone() {
        let original = image(400, 200);
        let smaller = reduced(&original, 100, Alpha::Perhaps).expect("it does not fit");

        assert_eq!((smaller.width(), smaller.height()), (100, 50));
        assert_eq!((original.width(), original.height()), (400, 200));
    }

    #[test]
    fn an_image_that_already_fits_has_no_reduced_copy() {
        assert!(reduced(&image(40, 20), 100, Alpha::Perhaps).is_none());
    }

    #[test]
    fn transparency_is_noticed_only_where_it_is() {
        assert!(!has_transparency(&image(8, 8)));

        let mut transparent = image(8, 8);
        transparent.put_pixel(7, 7, Rgba([1, 2, 3, 254]));
        assert!(has_transparency(&transparent));
    }

    /// The halo this is for: a soft edge over colour that is not meant to be
    /// seen. Resampling without premultiplying drags that colour into the
    /// pixels next to it.
    #[test]
    fn colour_under_transparent_pixels_does_not_bleed() {
        // A white square on the left, fully transparent black on the right.
        let mut original = RgbaImage::new(64, 8);
        for (x, _y, pixel) in original.enumerate_pixels_mut() {
            *pixel = if x < 32 {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([0, 0, 0, 0])
            };
        }

        let smaller = reduced(&original, 8, Alpha::Perhaps).expect("it does not fit");

        // Every pixel with any opacity left must still be white, rather than
        // having taken on the black hiding behind the transparent half.
        for pixel in smaller.pixels().filter(|pixel| pixel.0[3] > 8) {
            assert!(
                pixel.0[0] > 200,
                "the transparent black bled in: {:?}",
                pixel.0
            );
        }
    }

    #[test]
    fn preserves_colour() {
        let resized = to_max_edge(image(40, 40), Some(10), Alpha::Opaque);
        assert_eq!(resized.get_pixel(5, 5), &Rgba([12, 34, 56, 255]));
    }
}

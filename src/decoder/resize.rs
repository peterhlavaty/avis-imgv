//! Downscaling, using SIMD accelerated resampling.

use fast_image_resize::images::{Image as FirImage, ImageRef};
use fast_image_resize::{PixelType, ResizeOptions, Resizer};
use image::RgbaImage;

/// Shrinks `image` so its longest edge is at most `max_edge`.
///
/// Images already within the limit are returned untouched, and upscaling is
/// never done here: enlarging is the GPU's job at draw time.
pub fn to_max_edge(image: RgbaImage, max_edge: Option<u32>) -> RgbaImage {
    match max_edge.and_then(|edge| reduced(&image, edge)) {
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
pub fn reduced(image: &RgbaImage, max_edge: u32) -> Option<RgbaImage> {
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

    // Our alpha is opaque for every format that matters here, and the
    // premultiply round trip costs two full passes over the pixels.
    //
    // The filter is left at Lanczos3. Reducing a 24 megapixel photograph is
    // bound by reading it, not by the filter: measured against CatmullRom,
    // Bilinear and super sampling, none moved throughput, so the sharpest one
    // wins by default.
    let options = ResizeOptions::new().use_alpha(false);

    Resizer::new()
        .resize(&source, &mut destination, &options)
        .map_err(|e| tracing::error!("Failure resizing image -> {e}"))
        .ok()?;

    RgbaImage::from_raw(width, height, destination.into_vec())
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
        let resized = to_max_edge(image(400, 200), Some(100));
        assert_eq!((resized.width(), resized.height()), (100, 50));

        let resized = to_max_edge(image(200, 400), Some(100));
        assert_eq!((resized.width(), resized.height()), (50, 100));
    }

    #[test]
    fn never_upscales() {
        let resized = to_max_edge(image(40, 20), Some(100));
        assert_eq!((resized.width(), resized.height()), (40, 20));
    }

    #[test]
    fn no_limit_is_a_no_op() {
        let resized = to_max_edge(image(40, 20), None);
        assert_eq!((resized.width(), resized.height()), (40, 20));
    }

    #[test]
    fn extreme_ratios_keep_at_least_one_pixel() {
        let resized = to_max_edge(image(1000, 1), Some(10));
        assert_eq!((resized.width(), resized.height()), (10, 1));
    }

    #[test]
    fn a_reduced_copy_leaves_the_original_alone() {
        let original = image(400, 200);
        let smaller = reduced(&original, 100).expect("it does not fit");

        assert_eq!((smaller.width(), smaller.height()), (100, 50));
        assert_eq!((original.width(), original.height()), (400, 200));
    }

    #[test]
    fn an_image_that_already_fits_has_no_reduced_copy() {
        assert!(reduced(&image(40, 20), 100).is_none());
    }

    #[test]
    fn preserves_colour() {
        let resized = to_max_edge(image(40, 40), Some(10));
        assert_eq!(resized.get_pixel(5, 5), &Rgba([12, 34, 56, 255]));
    }
}

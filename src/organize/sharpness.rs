//! How sharp a photograph is, from the thumbnail the folder scan already has.
//!
//! Choosing between five frames of the same thing is mostly one question:
//! which of them is in focus. It is the question a contact sheet is worst at
//! answering — at thumbnail size everything looks acceptable — and the reason
//! people go through a burst one frame at a time at 100%.
//!
//! A number cannot answer it outright. A photograph of a wall is sharper by
//! any measure than a portrait at f/1.4 with a soft background, and the
//! portrait is the keeper. What a number does well is rank *frames of the same
//! scene*: five exposures of one subject, a second apart, differ in
//! sharpness for exactly one reason, and the sharpest of those is nearly
//! always the one to keep. So this is used to order the frames inside a group
//! and offered as a sort key, and it is never used to decide anything by
//! itself.
//!
//! The measure is Tenengrad — the mean squared Sobel gradient — which is the
//! one the autofocus literature settles on for exactly this comparison. It
//! runs over the camera's thumbnail rather than the photograph: the scan has
//! already decoded that, at a few hundred pixels a side, and reading a folder
//! of raw files again at full resolution to answer a question about relative
//! focus would cost minutes to change the answer very little.

use image::RgbaImage;

/// How sharp one photograph looked, as a number with no unit.
///
/// Comparable between photographs of the same scene at the same size, and not
/// otherwise: it is a ranking, not a measurement.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Sharpness(f32);

impl Sharpness {
    pub fn value(self) -> f32 {
        self.0
    }

    /// A number to show beside a thumbnail.
    ///
    /// The square root rather than the raw figure, which is a mean of squared
    /// gradients and reads as noise. To one decimal place, because the whole
    /// point is comparing it with the frame next to it: three frames of one
    /// burst can easily agree to the nearest whole number while one of them is
    /// genuinely the sharpest, and a marker on a frame whose number looks
    /// identical to its neighbour's is a marker nobody believes.
    pub fn score(self) -> f32 {
        (self.0.sqrt() * 10.0).round() / 10.0
    }
}

/// Measures `image`, or `None` when there is nothing to measure.
///
/// Nothing to measure is a real answer: a file with no embedded thumbnail has
/// no cheap way to be ranked, and guessing would put it in the wrong place in
/// a sorted list.
pub fn measure(image: &RgbaImage) -> Option<Sharpness> {
    let (width, height) = (image.width() as usize, image.height() as usize);

    // Sobel needs a pixel either side, so anything this small has no interior.
    if width < 3 || height < 3 {
        return None;
    }

    let luma = brightness(image, width, height);
    let mut total = 0.0f64;

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let at = |dx: usize, dy: usize| luma[(y + dy - 1) * width + (x + dx - 1)];

            // The two Sobel kernels, written out: this is the inner loop over
            // every pixel of every thumbnail in a folder.
            let horizontal =
                (at(2, 0) + 2.0 * at(2, 1) + at(2, 2)) - (at(0, 0) + 2.0 * at(0, 1) + at(0, 2));
            let vertical =
                (at(0, 2) + 2.0 * at(1, 2) + at(2, 2)) - (at(0, 0) + 2.0 * at(1, 0) + at(2, 0));

            total += f64::from(horizontal * horizontal + vertical * vertical);
        }
    }

    let interior = ((width - 2) * (height - 2)) as f64;

    Some(Sharpness((total / interior) as f32))
}

/// Perceived brightness, which is what focus is judged on.
fn brightness(image: &RgbaImage, width: usize, height: usize) -> Vec<f32> {
    let mut luma = Vec::with_capacity(width * height);

    for pixel in image.pixels() {
        let [r, g, b, _] = pixel.0;
        luma.push(0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b));
    }

    luma
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    /// A flat field: no edges anywhere, and so no sharpness at all.
    fn flat(width: u32, height: u32) -> RgbaImage {
        RgbaImage::from_pixel(width, height, Rgba([128, 128, 128, 255]))
    }

    /// Alternating columns, which is as much edge as an image can have.
    fn stripes(width: u32, height: u32, period: u32) -> RgbaImage {
        let mut image = RgbaImage::new(width, height);
        for (x, _y, pixel) in image.enumerate_pixels_mut() {
            let value = if (x / period).is_multiple_of(2) {
                0
            } else {
                255
            };
            *pixel = Rgba([value, value, value, 255]);
        }

        image
    }

    /// A soft gradient: edges everywhere but gentle ones.
    fn gradient(width: u32, height: u32) -> RgbaImage {
        let mut image = RgbaImage::new(width, height);
        for (x, _y, pixel) in image.enumerate_pixels_mut() {
            let value = ((x * 255) / width.max(1)) as u8;
            *pixel = Rgba([value, value, value, 255]);
        }

        image
    }

    #[test]
    fn a_flat_field_has_no_sharpness() {
        assert_eq!(measure(&flat(32, 32)).map(Sharpness::value), Some(0.0));
    }

    /// The property the whole thing rests on: a blurrier frame of the same
    /// scene scores lower.
    #[test]
    fn a_softer_picture_scores_lower_than_a_crisp_one() {
        let crisp = measure(&stripes(64, 64, 2)).expect("measurable");
        let soft = measure(&gradient(64, 64)).expect("measurable");
        let flat = measure(&flat(64, 64)).expect("measurable");

        assert!(
            crisp.value() > soft.value(),
            "crisp {} should beat soft {}",
            crisp.value(),
            soft.value()
        );
        assert!(
            soft.value() > flat.value(),
            "soft {} should beat flat {}",
            soft.value(),
            flat.value()
        );
    }

    /// And blurring one actually lowers it, which is the same claim made
    /// against a real blur rather than against a different picture.
    #[test]
    fn blurring_a_picture_lowers_its_score() {
        let sharp = stripes(64, 64, 2);
        let blurred = image::imageops::blur(&sharp, 2.0);

        let before = measure(&sharp).expect("measurable");
        let after = measure(&blurred).expect("measurable");

        assert!(
            after.value() < before.value(),
            "blurred {} should be under {}",
            after.value(),
            before.value()
        );
    }

    /// The score is the number a person sees, so it has to order the same way
    /// as the value it comes from.
    #[test]
    fn the_score_orders_the_same_way_as_the_value() {
        let crisp = measure(&stripes(64, 64, 2)).expect("measurable");
        let soft = measure(&gradient(64, 64)).expect("measurable");

        assert!(crisp.score() > soft.score());
        assert_eq!(measure(&flat(64, 64)).expect("measurable").score(), 0.0);
    }

    /// Two frames that differ slightly must show different numbers, or the
    /// marker on the sharper one looks arbitrary.
    #[test]
    fn the_score_can_tell_close_frames_apart() {
        let sharp = stripes(64, 64, 3);
        let barely_softer = image::imageops::blur(&sharp, 0.4);

        let a = measure(&sharp).expect("measurable");
        let b = measure(&barely_softer).expect("measurable");

        assert!(a.value() > b.value(), "{} vs {}", a.value(), b.value());
        assert_ne!(a.score(), b.score(), "both showed {}", a.score());
    }

    /// A pattern at exactly the sampling limit reads as flat, because a three
    /// tap gradient compares the pixel either side and those are the same one
    /// period apart. Worth knowing rather than worth fixing: it is a property
    /// of every gradient measure, it cannot happen to a photograph of anything
    /// real at thumbnail size, and the alternative is a wider kernel that
    /// blurs the very differences this exists to find.
    #[test]
    fn a_pattern_at_the_sampling_limit_reads_as_flat() {
        assert_eq!(
            measure(&stripes(64, 64, 1)).map(Sharpness::value),
            Some(0.0)
        );
    }

    /// Nothing to measure is a real answer rather than a zero, which would
    /// sort a file with no thumbnail in among the blurred ones.
    #[test]
    fn something_too_small_to_measure_says_so() {
        assert!(measure(&flat(2, 2)).is_none());
        assert!(measure(&flat(1, 100)).is_none());
        assert!(measure(&flat(3, 3)).is_some());
    }

    /// Two copies of the same picture score the same, so a ranking is stable.
    #[test]
    fn the_same_picture_scores_the_same() {
        let one = stripes(48, 48, 3);
        let other = stripes(48, 48, 3);

        assert_eq!(measure(&one), measure(&other));
    }
}

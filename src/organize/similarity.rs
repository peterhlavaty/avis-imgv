//! Telling whether two photographs show the same thing.
//!
//! Timestamps alone cannot separate a three frame bracket from three unrelated
//! pictures taken in the same few seconds, so the grouping also looks at what
//! the frames contain. Not at the pixels — at a sixty-four bit summary of the
//! camera's own thumbnail, which is already in memory by the time the question
//! is asked.
//!
//! The summary is a difference hash: shrink to a nine by eight grid of
//! brightness and record, for each of the sixty-four adjacent pairs, whether
//! the left cell is brighter than the right. What survives is the shape of the
//! scene. Two frames of a bracket differ by a stop of exposure and match
//! almost exactly; two frames of a burst of a moving subject match closely; a
//! different picture does not match at all.

use image::RgbaImage;

/// Cells across. One more than the bits, because each bit is a comparison
/// between neighbours.
const WIDTH: usize = 9;
/// Rows down, which with eight comparisons each makes sixty-four bits.
const HEIGHT: usize = 8;

/// A sixty-four bit summary of what a photograph looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fingerprint(u64);

impl Fingerprint {
    /// A fingerprint from its bits, for tests that want a known scene.
    #[cfg(test)]
    pub fn from_bits(bits: u64) -> Fingerprint {
        Fingerprint(bits)
    }

    /// How many of the sixty-four comparisons the two disagree on.
    ///
    /// Zero is the same picture. Under about ten is the same scene. Two
    /// unrelated photographs sit near thirty-two, which is what chance gives.
    pub fn distance(self, other: Fingerprint) -> u32 {
        (self.0 ^ other.0).count_ones()
    }

    /// Whether both frames show the same thing, within `tolerance`.
    pub fn resembles(self, other: Fingerprint, tolerance: u32) -> bool {
        self.distance(other) <= tolerance
    }
}

/// Summarises an image, which in practice is a camera's embedded thumbnail.
///
/// `None` for an image with no pixels in it, which is the only case where the
/// answer would be meaningless rather than merely uninteresting.
pub fn fingerprint(image: &RgbaImage) -> Option<Fingerprint> {
    if image.width() == 0 || image.height() == 0 {
        return None;
    }

    let cells = brightness_grid(image);
    let mut bits: u64 = 0;

    for row in 0..HEIGHT {
        for column in 0..WIDTH - 1 {
            let left = cells[row * WIDTH + column];
            let right = cells[row * WIDTH + column + 1];

            bits <<= 1;
            bits |= u64::from(left > right);
        }
    }

    Some(Fingerprint(bits))
}

/// Averages the image down to a [`WIDTH`] by [`HEIGHT`] grid of brightness.
///
/// A box average rather than a filtered resample: the grid is nine cells wide,
/// every source pixel lands in exactly one of them, and anything cleverer
/// would be measuring the resampler rather than the photograph.
fn brightness_grid(image: &RgbaImage) -> Vec<u32> {
    let mut totals = vec![0u64; WIDTH * HEIGHT];
    let mut counts = vec![0u64; WIDTH * HEIGHT];

    let (width, height) = (image.width() as usize, image.height() as usize);

    for (x, y, pixel) in image.enumerate_pixels() {
        let column = (x as usize * WIDTH / width).min(WIDTH - 1);
        let row = (y as usize * HEIGHT / height).min(HEIGHT - 1);
        let cell = row * WIDTH + column;

        totals[cell] += u64::from(luma(pixel.0));
        counts[cell] += 1;
    }

    totals
        .iter()
        .zip(&counts)
        .map(|(total, count)| (total / (*count).max(1)) as u32)
        .collect()
}

/// Perceived brightness, in the usual weights.
fn luma([red, green, blue, _]: [u8; 4]) -> u32 {
    (u32::from(red) * 299 + u32::from(green) * 587 + u32::from(blue) * 114) / 1000
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    /// An image whose brightness follows `shade(x, y)`.
    fn drawn(width: u32, height: u32, shade: impl Fn(u32, u32) -> u8) -> RgbaImage {
        RgbaImage::from_fn(width, height, |x, y| {
            let value = shade(x, y);
            Rgba([value, value, value, 255])
        })
    }

    fn of(image: &RgbaImage) -> Fingerprint {
        fingerprint(image).expect("an image with pixels has a fingerprint")
    }

    #[test]
    fn the_same_picture_matches_itself_exactly() {
        let image = drawn(160, 120, |x, y| ((x * 3 + y * 5) % 256) as u8);

        assert_eq!(of(&image).distance(of(&image)), 0);
    }

    #[test]
    fn a_darker_frame_of_the_same_scene_still_matches() {
        // A bracket: the same picture, two stops down. Every cell moves, but
        // the comparisons between neighbours do not.
        let bright = drawn(160, 120, |x, y| (40 + (x + y * 2) % 200) as u8);
        let dark = drawn(160, 120, |x, y| ((40 + (x + y * 2) % 200) / 4) as u8);

        assert!(
            of(&bright).resembles(of(&dark), 6),
            "{}",
            of(&bright).distance(of(&dark))
        );
    }

    #[test]
    fn a_different_scene_does_not_match() {
        let one = drawn(160, 120, |x, _| (x * 2 % 256) as u8);
        let other = drawn(160, 120, |_, y| (255 - (y * 2 % 256)) as u8);

        assert!(of(&one).distance(of(&other)) > 12);
    }

    #[test]
    fn a_frame_of_a_burst_matches_the_one_before_it() {
        // The same scene with the subject a few pixels along, which is what a
        // burst of a moving subject looks like.
        let first = drawn(160, 120, |x, y| ((x / 8 + y / 8) * 20 % 256) as u8);
        let second = drawn(160, 120, |x, y| (((x + 3) / 8 + y / 8) * 20 % 256) as u8);

        assert!(of(&first).resembles(of(&second), 10));
    }

    #[test]
    fn the_size_of_the_thumbnail_does_not_change_the_answer() {
        // Cameras write thumbnails of whatever size they like, and two files
        // from the same shoot have to compare even so.
        let shade = |x: u32, y: u32| ((x * 256 / 160 + y * 256 / 120) % 256) as u8;

        let large = drawn(160, 120, shade);
        let small = drawn(80, 60, |x, y| shade(x * 2, y * 2));

        assert!(of(&large).resembles(of(&small), 8));
    }

    #[test]
    fn an_image_with_no_pixels_has_no_fingerprint() {
        assert!(fingerprint(&RgbaImage::new(0, 0)).is_none());
    }

    #[test]
    fn a_flat_image_is_a_fingerprint_rather_than_a_failure() {
        // Nothing is brighter than its neighbour, so every bit is zero. Two
        // blank frames match each other, which is the right answer.
        let grey = drawn(64, 64, |_, _| 128);

        assert_eq!(of(&grey).distance(of(&grey)), 0);
    }

    #[test]
    fn a_thumbnail_smaller_than_the_grid_still_works() {
        let tiny = drawn(4, 3, |x, y| ((x + y) * 30) as u8);

        assert_eq!(fingerprint(&tiny).map(|f| f.distance(f)), Some(0));
    }

    #[test]
    fn distance_is_symmetric_and_bounded() {
        let one = drawn(64, 64, |x, _| (x * 4 % 256) as u8);
        let other = drawn(64, 64, |_, y| (y * 4 % 256) as u8);

        let (a, b) = (of(&one), of(&other));

        assert_eq!(a.distance(b), b.distance(a));
        assert!(a.distance(b) <= 64);
    }
}

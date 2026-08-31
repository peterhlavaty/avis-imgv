//! How the tones of a photograph are distributed, and how much of it is gone.
//!
//! A histogram is table stakes for a viewer somebody culls in: it is how you
//! tell an underexposed frame from a dark subject, and it is the only reliable
//! way to see that a highlight has clipped, because a screen shows 250 and 255
//! as the same white.
//!
//! What makes it worth more here than in a single-file viewer is where it is
//! computed. The decode workers already touch every pixel of every photograph
//! in the folder — that is what decoding is — so accumulating four counters
//! while the pixels are in cache turns "what proportion of this frame is
//! blown" into something known for the whole folder rather than for the frame
//! on screen. That makes it a filter and a sort key, which no viewer that
//! computes its histogram when you look at a picture can offer.
//!
//! Counted on the reduced copy that goes to the screen rather than the full
//! resolution one. A histogram is a shape and a proportion, and neither moves
//! meaningfully between a photograph and the same photograph at a third the
//! size — while the difference in cost is the whole reason this is affordable.

/// Rec.709 luminance, as 8.8 fixed point: 0.2126, 0.7152 and 0.0722 times
/// 256, rounded so that they still sum to exactly 256.
///
/// Exactly, because a sum of 257 would let a white pixel land one bucket past
/// the end of the array.
const LUMA_RED: u32 = 54;
const LUMA_GREEN: u32 = 183;
const LUMA_BLUE: u32 = 19;

/// How many buckets each channel is counted into.
///
/// One per possible value: the pixels are eight bit, so anything coarser would
/// be throwing away detail to save an array smaller than a thumbnail.
pub const BUCKETS: usize = 256;

/// The tones of one photograph.
#[derive(Debug, Clone)]
pub struct Histogram {
    /// Counts per value, for red, green, blue and perceived brightness.
    pub red: Box<[u32; BUCKETS]>,
    pub green: Box<[u32; BUCKETS]>,
    pub blue: Box<[u32; BUCKETS]>,
    pub luma: Box<[u32; BUCKETS]>,
    /// How many pixels went into it.
    pub pixels: u32,
    /// Pixels with any channel at the very top, and at the very bottom.
    ///
    /// Any channel rather than all of them: a sky that has clipped only in
    /// blue has lost its detail there just as surely, and a viewer that only
    /// counted white would say nothing about it.
    pub blown: u32,
    pub crushed: u32,
}

impl Default for Histogram {
    fn default() -> Self {
        Histogram {
            red: Box::new([0; BUCKETS]),
            green: Box::new([0; BUCKETS]),
            blue: Box::new([0; BUCKETS]),
            luma: Box::new([0; BUCKETS]),
            pixels: 0,
            blown: 0,
            crushed: 0,
        }
    }
}

impl Histogram {
    /// Counts the tones of `pixels`, which are RGBA, four bytes each.
    pub fn of(pixels: &[u8]) -> Histogram {
        let mut found = Histogram::default();

        for pixel in pixels.as_chunks::<4>().0 {
            let [r, g, b, _] = *pixel;

            found.red[r as usize] += 1;
            found.green[g as usize] += 1;
            found.blue[b as usize] += 1;

            // The Rec.709 weights, in fixed point. The eye is not equally
            // sensitive to the three, and this runs once per pixel of every
            // photograph in the folder — the floating point version of the
            // same sum, with a round, cost fifteen per cent of the viewer's
            // throughput. These weights sum to 256 exactly, so the shift is
            // the division and the result cannot leave the range.
            let luma =
                (LUMA_RED * u32::from(r) + LUMA_GREEN * u32::from(g) + LUMA_BLUE * u32::from(b))
                    >> 8;
            found.luma[luma as usize] += 1;

            if r == 255 || g == 255 || b == 255 {
                found.blown += 1;
            }
            if r == 0 && g == 0 && b == 0 {
                found.crushed += 1;
            }

            found.pixels += 1;
        }

        found
    }

    /// What proportion of the photograph has clipped highlights, as a
    /// percentage.
    pub fn blown_percent(&self) -> f32 {
        self.percent(self.blown)
    }

    /// And crushed shadows.
    pub fn crushed_percent(&self) -> f32 {
        self.percent(self.crushed)
    }

    fn percent(&self, count: u32) -> f32 {
        if self.pixels == 0 {
            return 0.0;
        }

        count as f32 * 100.0 / self.pixels as f32
    }

    /// The tallest bucket in any channel, which is what the drawing scales to.
    ///
    /// The whole set rather than per channel: three curves each normalised to
    /// their own peak would show a colour cast as three identical shapes.
    pub fn tallest(&self) -> u32 {
        [&self.red, &self.green, &self.blue]
            .into_iter()
            .flat_map(|channel| channel.iter().copied())
            .max()
            .unwrap_or(0)
    }

    /// Whether anything was counted at all.
    pub fn is_empty(&self) -> bool {
        self.pixels == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `count` pixels of one colour, as the decoder would hand them over.
    fn pixels(colour: [u8; 4], count: usize) -> Vec<u8> {
        colour.iter().copied().cycle().take(count * 4).collect()
    }

    #[test]
    fn nothing_counts_as_nothing() {
        let found = Histogram::of(&[]);

        assert!(found.is_empty());
        assert_eq!(found.blown_percent(), 0.0);
        assert_eq!(found.crushed_percent(), 0.0);
        assert_eq!(found.tallest(), 0);
    }

    #[test]
    fn every_pixel_lands_in_its_own_bucket() {
        let found = Histogram::of(&pixels([10, 20, 30, 255], 7));

        assert_eq!(found.pixels, 7);
        assert_eq!(found.red[10], 7);
        assert_eq!(found.green[20], 7);
        assert_eq!(found.blue[30], 7);
        assert_eq!(found.red[11], 0);
    }

    /// The measurement that makes this worth having: how much of the frame is
    /// past saving.
    #[test]
    fn clipping_is_counted_at_both_ends() {
        let mut data = pixels([255, 255, 255, 255], 1);
        data.extend(pixels([0, 0, 0, 255], 1));
        data.extend(pixels([128, 128, 128, 255], 2));

        let found = Histogram::of(&data);

        assert_eq!(found.blown, 1);
        assert_eq!(found.crushed, 1);
        assert_eq!(found.blown_percent(), 25.0);
        assert_eq!(found.crushed_percent(), 25.0);
    }

    /// A sky clipped only in blue has lost its detail there just as surely,
    /// and a viewer that only counted white would say nothing about it.
    #[test]
    fn a_single_channel_clipping_still_counts() {
        let found = Histogram::of(&pixels([100, 100, 255, 255], 4));

        assert_eq!(found.blown, 4);
        assert_eq!(found.blown_percent(), 100.0);
    }

    /// And a shadow is only crushed when there is nothing left in any of
    /// them: a deep blue with no red is not a black pixel.
    #[test]
    fn a_dark_colour_is_not_a_crushed_shadow() {
        let found = Histogram::of(&pixels([0, 0, 40, 255], 4));

        assert_eq!(found.crushed, 0);
    }

    /// The fixed point weights have to sum to exactly one unit, or a white
    /// pixel lands past the end of the array.
    #[test]
    fn the_luma_weights_cannot_overflow_the_buckets() {
        assert_eq!(LUMA_RED + LUMA_GREEN + LUMA_BLUE, 256);

        let white = Histogram::of(&pixels([255, 255, 255, 255], 1));
        assert_eq!(white.luma[BUCKETS - 1], 1);

        let black = Histogram::of(&pixels([0, 0, 0, 255], 1));
        assert_eq!(black.luma[0], 1);
    }

    /// And they still say what the floating point weights said, to within a
    /// level or so, which is all a 256 bucket histogram can show anyway.
    #[test]
    fn the_fixed_point_weights_agree_with_the_real_ones() {
        for (r, g, b) in [
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (128, 64, 200),
            (17, 200, 90),
        ] {
            let found = Histogram::of(&pixels([r, g, b, 255], 1));
            let at = found
                .luma
                .iter()
                .position(|count| *count > 0)
                .expect("a bucket");

            let exact = 0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b);

            assert!(
                (at as f32 - exact).abs() <= 1.5,
                "{r},{g},{b}: bucket {at} against {exact}"
            );
        }
    }

    #[test]
    fn brightness_is_weighted_the_way_the_eye_is() {
        // Pure green reads far brighter than pure blue at the same value.
        let green = Histogram::of(&pixels([0, 255, 0, 255], 1));
        let blue = Histogram::of(&pixels([0, 0, 255, 255], 1));

        let peak = |found: &Histogram| {
            found
                .luma
                .iter()
                .enumerate()
                .max_by_key(|(_, count)| **count)
                .map(|(at, _)| at)
                .unwrap_or(0)
        };

        assert!(peak(&green) > peak(&blue), "green should read brighter");
    }

    /// One scale for all three channels, or a colour cast would be drawn as
    /// three identical curves.
    #[test]
    fn the_scale_is_the_tallest_of_any_channel() {
        let mut data = pixels([10, 10, 10, 255], 5);
        data.extend(pixels([20, 200, 20, 255], 9));

        let found = Histogram::of(&data);

        assert_eq!(found.tallest(), 9);
    }

    #[test]
    fn a_trailing_part_pixel_is_ignored() {
        // Three and a half pixels' worth of bytes.
        let found = Histogram::of(&[255, 255, 255, 255, 0, 0]);

        assert_eq!(found.pixels, 1);
    }
}

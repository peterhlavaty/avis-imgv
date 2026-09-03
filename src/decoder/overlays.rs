//! The two things a photograph will not tell you by being looked at.
//!
//! A screen shows 250 and 255 as the same white, so a blown highlight is
//! invisible until it is marked. And at anything under 100% a slightly missed
//! focus looks exactly like a hit one, which is why people zoom into every
//! frame of a burst one at a time.
//!
//! Both are answered by painting over the picture: the clipped pixels in a
//! colour nothing photographic is, and the in-focus edges likewise. Every
//! camera with a decent live view does the second under the name focus
//! peaking, and every raw converter does the first.
//!
//! Built as an image the same size as the copy on screen, mostly transparent,
//! and drawn with the photograph's own texture coordinates — so it follows the
//! zoom and the pan for nothing, and a quarter turn turns it too. Built when
//! the overlay is switched on rather than on every decode, because it is a
//! full pass over the pixels for a question nobody is asking most of the time.

use image::{Rgba, RgbaImage};

/// What is being marked.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Overlay {
    #[default]
    Off,
    /// Highlights that have clipped, and shadows that have gone.
    Clipping,
    /// The edges that are in focus.
    Peaking,
}

impl Overlay {
    pub const ALL: &'static [Overlay] = &[Overlay::Off, Overlay::Clipping, Overlay::Peaking];

    pub fn label(self) -> &'static str {
        match self {
            Overlay::Off => "Off",
            Overlay::Clipping => "Clipping",
            Overlay::Peaking => "Focus peaking",
        }
    }

    /// The next one round, for the key that cycles them.
    pub fn next(self) -> Overlay {
        match self {
            Overlay::Off => Overlay::Clipping,
            Overlay::Clipping => Overlay::Peaking,
            Overlay::Peaking => Overlay::Off,
        }
    }
}

/// Red for a blown highlight, blue for a crushed shadow.
///
/// The two colours every raw converter uses for this, and both are chosen for
/// being obviously not photographic at full saturation.
const BLOWN: Rgba<u8> = Rgba([255, 40, 40, 220]);
const CRUSHED: Rgba<u8> = Rgba([60, 120, 255, 220]);

/// The colour focus peaking marks an edge in.
const PEAK: Rgba<u8> = Rgba([120, 255, 90, 235]);

/// Nothing at all, which is most of the mask.
const CLEAR: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// What share of the photograph focus peaking marks.
///
/// A proportion rather than a fixed gradient, which is the whole difficulty
/// with this. A threshold that marks a portrait's eyelashes and nothing else
/// marks a hillside of foliage entirely — tried at a fixed value first, and a
/// detailed outdoor frame came back solid green, which tells nobody anything.
/// The question is not "how sharp is this edge" but "which of this
/// photograph's edges are the sharpest", so the threshold is read off the
/// picture's own distribution.
///
/// A twentieth is what the cameras that do this settle on: enough to trace
/// the plane of focus, little enough to still see the photograph under it.
const MARKED_SHARE: f32 = 0.05;

/// How many buckets the gradients are counted into to find that threshold.
const GRADIENT_BUCKETS: usize = 512;

/// Builds the mask for `pixels`, which are RGBA and `width` by `height`.
pub fn mask(overlay: Overlay, pixels: &[u8], width: u32, height: u32) -> Option<RgbaImage> {
    match overlay {
        Overlay::Off => None,
        Overlay::Clipping => Some(clipping(pixels, width, height)),
        Overlay::Peaking => peaking(pixels, width, height),
    }
}

/// Marks what has clipped at each end.
fn clipping(pixels: &[u8], width: u32, height: u32) -> RgbaImage {
    let mut mask = RgbaImage::from_pixel(width, height, CLEAR);

    for (at, pixel) in pixels.as_chunks::<4>().0.iter().enumerate() {
        let [r, g, b, _] = *pixel;

        // The same rule the histogram counts by, so the picture and the number
        // beside it cannot disagree.
        let colour = if r == 255 || g == 255 || b == 255 {
            BLOWN
        } else if r == 0 && g == 0 && b == 0 {
            CRUSHED
        } else {
            continue;
        };

        let (x, y) = ((at as u32) % width, (at as u32) / width);
        if y < height {
            mask.put_pixel(x, y, colour);
        }
    }

    mask
}

/// Marks the edges that are in focus.
///
/// Two passes: the gradient of every pixel, then the value only a twentieth of
/// them exceed, then the marking. The second pass is what makes it work on any
/// photograph rather than only on the ones whose contrast happens to suit a
/// number chosen in advance.
fn peaking(pixels: &[u8], width: u32, height: u32) -> Option<RgbaImage> {
    let (w, h) = (width as usize, height as usize);
    if w < 3 || h < 3 {
        return None;
    }

    // Brightness once, rather than three channels three times: focus is a
    // question about detail, and detail is in the luminance.
    let luma: Vec<f32> = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|[r, g, b, _]| {
            0.2126 * f32::from(*r) + 0.7152 * f32::from(*g) + 0.0722 * f32::from(*b)
        })
        .collect();

    if luma.len() < w * h {
        return None;
    }

    let gradients = gradients(&luma, w, h);
    let threshold = strongest(&gradients, MARKED_SHARE)?;

    let mut mask = RgbaImage::from_pixel(width, height, CLEAR);

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            if gradients[y * w + x] >= threshold {
                mask.put_pixel(x as u32, y as u32, PEAK);
            }
        }
    }

    Some(mask)
}

/// The Sobel magnitude at every pixel, with the border left at nothing.
fn gradients(luma: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut found = vec![0.0f32; w * h];

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let at = |dx: usize, dy: usize| luma[(y + dy - 1) * w + (x + dx - 1)];

            let horizontal =
                (at(2, 0) + 2.0 * at(2, 1) + at(2, 2)) - (at(0, 0) + 2.0 * at(0, 1) + at(0, 2));
            let vertical =
                (at(0, 2) + 2.0 * at(1, 2) + at(2, 2)) - (at(0, 0) + 2.0 * at(1, 0) + at(2, 0));

            found[y * w + x] = (horizontal * horizontal + vertical * vertical).sqrt();
        }
    }

    found
}

/// The gradient only `share` of the picture exceeds.
///
/// Counted into buckets rather than sorted: sorting two megapixels of floats
/// to find one number would cost more than everything else here put together.
///
/// `None` when the photograph has no edges worth marking at all — a blank wall
/// or a frame so far out of focus that nothing stands out. Marking the
/// strongest twentieth of nothing would draw a green haze over an empty
/// picture and claim it was in focus.
fn strongest(gradients: &[f32], share: f32) -> Option<f32> {
    let tallest = gradients.iter().copied().fold(0.0f32, f32::max);
    if tallest <= f32::EPSILON {
        return None;
    }

    let mut counts = [0usize; GRADIENT_BUCKETS];
    for gradient in gradients {
        let bucket = (gradient / tallest * (GRADIENT_BUCKETS - 1) as f32) as usize;
        counts[bucket.min(GRADIENT_BUCKETS - 1)] += 1;
    }

    // Down from the top until the wanted share has been accounted for.
    let wanted = (gradients.len() as f32 * share) as usize;
    let mut seen = 0usize;

    for (bucket, count) in counts.iter().enumerate().rev() {
        seen += count;
        if seen >= wanted.max(1) {
            return Some(bucket as f32 / (GRADIENT_BUCKETS - 1) as f32 * tallest);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(colour: [u8; 4], width: u32, height: u32) -> Vec<u8> {
        colour
            .iter()
            .copied()
            .cycle()
            .take((width * height * 4) as usize)
            .collect()
    }

    #[test]
    fn off_makes_no_mask_at_all() {
        assert!(mask(Overlay::Off, &flat([255, 255, 255, 255], 4, 4), 4, 4).is_none());
    }

    #[test]
    fn every_overlay_has_a_name_and_they_cycle() {
        let mut overlay = Overlay::default();
        let mut seen = vec![overlay];

        for _ in 0..Overlay::ALL.len() {
            overlay = overlay.next();
            assert!(!overlay.label().is_empty());
            if !seen.contains(&overlay) {
                seen.push(overlay);
            }
        }

        assert_eq!(seen.len(), Overlay::ALL.len());
        assert_eq!(overlay, Overlay::default(), "it comes back round");
    }

    /// Why a row offering to show the photograph as it is cannot use the
    /// key's cycle: one along from the clipping mask is another mask, and the
    /// status bar's word said it would show the photograph while turning
    /// focus peaking on.
    #[test]
    fn the_mask_after_the_clipping_one_is_not_nothing() {
        assert_eq!(Overlay::Clipping.next(), Overlay::Peaking);
        assert_ne!(Overlay::Clipping.next(), Overlay::Off);
    }

    /// The point of the clipping overlay: what a screen cannot show.
    #[test]
    fn clipping_marks_both_ends_and_nothing_between() {
        let mut pixels = flat([128, 128, 128, 255], 3, 1);
        pixels[0..4].copy_from_slice(&[255, 255, 255, 255]);
        pixels[8..12].copy_from_slice(&[0, 0, 0, 255]);

        let found = clipping(&pixels, 3, 1);

        assert_eq!(*found.get_pixel(0, 0), BLOWN);
        assert_eq!(*found.get_pixel(1, 0), CLEAR, "the mid tone was marked");
        assert_eq!(*found.get_pixel(2, 0), CRUSHED);
    }

    /// It marks by the same rule the histogram counts by, or the picture and
    /// the number beside it would disagree.
    #[test]
    fn a_single_channel_clipping_is_marked_like_the_histogram_counts_it() {
        let pixels = flat([100, 100, 255, 255], 2, 1);

        let found = clipping(&pixels, 2, 1);
        let counted = crate::decoder::histogram::Histogram::of(&pixels);

        assert_eq!(*found.get_pixel(0, 0), BLOWN);
        assert_eq!(counted.blown, 2);
    }

    /// An edge is marked and the flat either side of it is not.
    #[test]
    fn peaking_marks_an_edge() {
        let mut pixels = Vec::new();
        for _y in 0..8 {
            for x in 0..8u32 {
                let value = if x < 4 { 0u8 } else { 255 };
                pixels.extend_from_slice(&[value, value, value, 255]);
            }
        }

        let found = peaking(&pixels, 8, 8).expect("big enough");

        assert_eq!(*found.get_pixel(4, 4), PEAK, "the edge was not marked");
        assert_eq!(*found.get_pixel(1, 4), CLEAR, "the flat side was marked");
    }

    /// The failure a fixed threshold could not avoid: a detailed picture
    /// marked entirely, which tells nobody anything. A photograph has a spread
    /// of gradient strengths, and only the top of that spread is marked
    /// however contrasty the picture is.
    #[test]
    fn only_a_small_share_of_a_busy_picture_is_marked() {
        // A ramp everywhere, with a few hard edges in it: a spread of
        // gradients, which is what a photograph has.
        let mut pixels = Vec::new();
        for y in 0..64u32 {
            for x in 0..64u32 {
                let value = if x.is_multiple_of(16) {
                    255u8
                } else {
                    (x * 2) as u8
                };
                let _ = y;
                pixels.extend_from_slice(&[value, value, value, 255]);
            }
        }

        let found = peaking(&pixels, 64, 64).expect("big enough");
        let marked = found.pixels().filter(|pixel| **pixel == PEAK).count();
        let all = (64 * 64) as f32;

        assert!(
            marked as f32 / all < 0.35,
            "{marked} of {all} pixels marked"
        );
    }

    /// The one case the proportion cannot help with, written down rather than
    /// pretended away: a picture where a huge share of the pixels sit at
    /// exactly the same maximum gradient has no "strongest twentieth" to find,
    /// so all of them are marked. It takes a synthetic checkerboard to build —
    /// no photograph of anything has a gradient histogram like that.
    #[test]
    fn a_picture_of_nothing_but_identical_edges_is_marked_throughout() {
        let mut pixels = Vec::new();
        for y in 0..32u32 {
            for x in 0..32u32 {
                let value = if ((x / 2) + (y / 2)).is_multiple_of(2) {
                    0u8
                } else {
                    255
                };
                pixels.extend_from_slice(&[value, value, value, 255]);
            }
        }

        let found = peaking(&pixels, 32, 32).expect("big enough");
        let marked = found.pixels().filter(|pixel| **pixel == PEAK).count();

        assert!(marked > (32 * 32) / 2, "only {marked} marked");
    }

    /// And the sharp part of a picture is what gets marked, rather than the
    /// soft part, which is the entire point.
    #[test]
    fn the_sharp_half_is_marked_and_the_soft_half_is_not() {
        // A hard edge on the left, a gentle ramp on the right.
        let mut pixels = Vec::new();
        for _y in 0..32u32 {
            for x in 0..32u32 {
                let value = if x < 16 {
                    if x < 8 {
                        0u8
                    } else {
                        255
                    }
                } else {
                    ((x - 16) * 8) as u8
                };
                pixels.extend_from_slice(&[value, value, value, 255]);
            }
        }

        let found = peaking(&pixels, 32, 32).expect("big enough");

        let marked_in = |from: u32, to: u32| {
            (from..to)
                .flat_map(|x| (1..31u32).map(move |y| (x, y)))
                .filter(|(x, y)| *found.get_pixel(*x, *y) == PEAK)
                .count()
        };

        assert!(
            marked_in(6, 10) > marked_in(20, 24),
            "the soft side was marked as much as the hard edge"
        );
    }

    /// A picture with nothing standing out is not given a green haze and told
    /// it is in focus.
    #[test]
    fn a_picture_with_no_edges_is_left_alone() {
        let pixels = flat([128, 128, 128, 255], 16, 16);
        let found = peaking(&pixels, 16, 16);

        assert!(
            found.is_none_or(|mask| mask.pixels().all(|pixel| *pixel == CLEAR)),
            "a flat picture was marked"
        );
    }

    #[test]
    fn something_too_small_to_peak_says_so() {
        assert!(peaking(&flat([128, 128, 128, 255], 2, 2), 2, 2).is_none());
    }
}

//! Fitting one rectangle inside another, keeping its shape.
//!
//! The oldest duplicated arithmetic in the program: five copies, in the
//! decoder, the canvas, the contact sheet and the folder jobs. They had drifted
//! in a way that shows on screen — given a photograph of no size, two of them
//! returned nothing and one returned the whole cell, so the same broken file
//! drew a blank space in one view and a full-cell smear of whatever the texture
//! last held in another.
//!
//! The best of the five was the integer one in the decoder: it is the only one
//! with a guard on a zero *bound* and a floor of one pixel, so a very wide
//! panorama cannot be scaled to a texture of no height. That is the version
//! this is, generalised.
//!
//! # Where it lives, and why not in the drawing layer
//!
//! Six things ask this question and only three of them draw: the decoder sizes
//! its output, the GPU cache clamps to the adapter's maximum texture edge, and
//! the folder jobs size their thumbnails. A helper lifted into `src/view/`
//! would have left the tested integer copy behind and given the crate two
//! answers again.
//!
//! # Where the toolkit's vector is taught to fit
//!
//! [`Edges`] is implemented for `(u32, u32)` and `(f32, f32)` here, and for the
//! toolkit's own vector in the drawing layer — `view::texture`.
//!
//! Be precise about what that buys, because it is easy to overstate. Within one
//! crate the trait is local everywhere, so the orphan rule does not forbid
//! writing that impl here; coherence only stops a *second* one. The placement
//! is therefore a convention today, and it becomes the compiler's rule on the
//! day this file is a crate of its own — which is the point of keeping it able
//! to be one. What is checked now is that there is exactly one implementation
//! and it is not in this file.

/// A pair of dimensions that can be measured and rebuilt.
///
/// Implemented for whatever a caller already holds, so nothing converts into a
/// type belonging to this module and back out again.
pub trait Edges: Copy {
    fn width(self) -> f32;
    fn height(self) -> f32;

    /// Builds one from a width and a height.
    ///
    /// Where the type is integral this is also where rounding and the
    /// one-pixel floor happen, which is why a panorama scaled to nothing
    /// cannot come out of here.
    fn of(width: f32, height: f32) -> Self;
}

impl Edges for (f32, f32) {
    fn width(self) -> f32 {
        self.0
    }

    fn height(self) -> f32 {
        self.1
    }

    fn of(width: f32, height: f32) -> Self {
        (width, height)
    }
}

impl Edges for (u32, u32) {
    fn width(self) -> f32 {
        self.0 as f32
    }

    fn height(self) -> f32 {
        self.1 as f32
    }

    /// Rounded, and never less than a pixel: a texture of no height is not a
    /// smaller picture, it is a crash on some drivers and an invisible one on
    /// others.
    fn of(width: f32, height: f32) -> Self {
        (
            (width.round().max(1.0)) as u32,
            (height.round().max(1.0)) as u32,
        )
    }
}

/// Whether something smaller than the space may be made bigger to fill it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grow {
    /// Never enlarge. A small photograph is shown at its own size until it is
    /// zoomed, because enlarging it is a decision the user has not made.
    Never,
    /// Scale up as well as down. What a raw file's embedded copy needs: some
    /// DNGs carry a 256 pixel preview and nothing else, and drawn at its own
    /// size it is a postage stamp in the middle of a 4K screen.
    ToFill,
}

/// The largest rectangle with `edges`' shape that fits inside `within`.
///
/// `None` where either rectangle has no area. That is deliberately not a size:
/// the callers disagree about what to draw for a photograph of no size — the
/// canvas draws nothing, a cell in the contact sheet draws its own outline —
/// and each of them should say so rather than inherit somebody else's answer
/// by accident, which is how they came to differ in the first place.
pub fn inside<E: Edges>(edges: E, within: E, grow: Grow) -> Option<E> {
    let (width, height) = (edges.width(), edges.height());
    let (room_x, room_y) = (within.width(), within.height());

    if width <= 0.0 || height <= 0.0 || room_x <= 0.0 || room_y <= 0.0 {
        return None;
    }

    let scale = (room_x / width).min(room_y / height);
    let scale = match grow {
        Grow::Never => scale.min(1.0),
        Grow::ToFill => scale,
    };

    Some(E::of(width * scale, height * scale))
}

/// The largest rectangle with `edges`' shape whose longest side is at most
/// `most`, or `None` where it already is.
///
/// The decoder's question: what to resample to. `None` means "no work to do"
/// rather than "no answer", which is why it is its own function — a caller
/// that resampled on `None` would rebuild every image at its own size.
///
/// A `most` of zero means no limit, which is how the configuration spells
/// "leave it alone".
pub fn longest_edge<E: Edges>(edges: E, most: u32) -> Option<E> {
    if most == 0 {
        return None;
    }

    let longest = edges.width().max(edges.height());
    if longest <= most as f32 {
        return None;
    }

    inside(edges, E::of(most as f32, most as f32), Grow::Never)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_picture_fits_by_its_width() {
        let fitted = inside((400.0, 200.0), (100.0, 100.0), Grow::Never);

        assert_eq!(fitted, Some((100.0, 50.0)));
    }

    #[test]
    fn a_tall_picture_fits_by_its_height() {
        let fitted = inside((200.0, 400.0), (100.0, 100.0), Grow::Never);

        assert_eq!(fitted, Some((50.0, 100.0)));
    }

    /// A small photograph is shown at its own size until it is zoomed.
    #[test]
    fn fitting_never_enlarges() {
        let fitted = inside((10.0, 10.0), (100.0, 100.0), Grow::Never);

        assert_eq!(fitted, Some((10.0, 10.0)));
    }

    /// What a 256 pixel embedded preview needs, so it is not a postage stamp
    /// in the middle of a 4K screen.
    #[test]
    fn filling_enlarges() {
        let filled = inside((10.0, 10.0), (100.0, 100.0), Grow::ToFill);

        assert_eq!(filled, Some((100.0, 100.0)));
    }

    /// The divergence this module exists to end: two of the five copies
    /// answered a degenerate picture with nothing and one with the whole cell,
    /// so the same broken file drew a blank space in one view and a smear in
    /// another. There is now one answer, and it is "you decide".
    #[test]
    fn a_picture_with_no_area_has_no_answer() {
        assert_eq!(inside((0.0, 100.0), (50.0, 50.0), Grow::Never), None);
        assert_eq!(inside((100.0, 0.0), (50.0, 50.0), Grow::Never), None);
        assert_eq!(inside((-1.0, 10.0), (50.0, 50.0), Grow::Never), None);
    }

    #[test]
    fn a_space_with_no_area_has_no_answer_either() {
        assert_eq!(inside((100.0, 100.0), (0.0, 50.0), Grow::Never), None);
        assert_eq!(inside((100.0, 100.0), (50.0, 0.0), Grow::Never), None);
    }

    /// The property the decoder's copy had and the four float ones did not: a
    /// texture of no height is a crash on some drivers and an invisible
    /// picture on others.
    #[test]
    fn an_extreme_panorama_keeps_at_least_one_pixel() {
        let fitted: Option<(u32, u32)> = inside((10_000, 1), (100, 100), Grow::Never);

        assert_eq!(fitted, Some((100, 1)));
    }

    #[test]
    fn the_decoder_s_question_is_answered_in_whole_pixels() {
        assert_eq!(longest_edge((4000u32, 3000u32), 1000), Some((1000, 750)));
    }

    /// `None` means "no work to do" — a caller that resampled on it would
    /// rebuild every photograph at its own size.
    #[test]
    fn a_picture_already_small_enough_needs_no_resampling() {
        assert_eq!(longest_edge((400u32, 300u32), 1000), None);
        assert_eq!(longest_edge((1000u32, 750u32), 1000), None);
    }

    /// How the configuration spells "leave it alone".
    #[test]
    fn a_bound_of_zero_is_no_bound() {
        assert_eq!(longest_edge((4000u32, 3000u32), 0), None);
    }

    #[test]
    fn the_two_kinds_of_edges_agree() {
        let floats = inside((400.0, 200.0), (100.0, 100.0), Grow::Never);
        let integers = inside((400u32, 200u32), (100u32, 100u32), Grow::Never);

        assert_eq!(floats, Some((100.0, 50.0)));
        assert_eq!(integers, Some((100, 50)));
    }
}

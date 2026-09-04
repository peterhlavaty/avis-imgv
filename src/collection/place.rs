//! Where a photograph was left: how far in, and whereabouts.
//!
//! Something each photograph carries, rather than something the view owns —
//! which is why it is here and not in `view::image_view`. The history
//! remembers it, and had to name a drawing module to do so.
//!
//! `Pan` is two numbers rather than the toolkit's vector. It writes as `[x, y]`,
//! which is exactly what the hand-written `serde` shim it replaces wrote, so
//! a `history.json` from before this change reads unchanged. The shim existed
//! because egui's own `serde` feature is not switched on in this build, and
//! switching it on to write two floats would pull serialisation into every
//! type the interface is made of.

/// How far the photograph has been moved from the middle, in points.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct Pan(pub f32, pub f32);

impl Pan {
    pub const NONE: Pan = Pan(0.0, 0.0);

    pub fn x(self) -> f32 {
        self.0
    }

    pub fn y(self) -> f32 {
        self.1
    }
}

/// The part of a viewport that belongs to an image rather than to the view.
///
/// The latches — whether new images should fill the panel — are a preference
/// and stay where they are; what is remembered is where the user got to.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub zoom: f32,
    pub pan: Pan,
}

impl Place {
    /// The whole image, centred: what an image that was never touched shows.
    pub const UNTOUCHED: Place = Place {
        zoom: 1.0,
        pan: Pan::NONE,
    };

    /// Whether this is worth a map entry.
    pub fn is_worth_remembering(&self) -> bool {
        // The pan is meaningless at a zoom that shows the whole image, and is
        // clamped away on the next frame anyway.
        (self.zoom - 1.0).abs() > f32::EPSILON
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `history.json` is persisted, so the shape this writes is not free to
    /// change. It replaced a hand-written `serde` shim that wrote a `Vec2` as
    /// `[x, y]`, and it must still write `[x, y]` or every file anybody has
    /// stops loading.
    #[test]
    fn a_pan_is_still_written_as_two_numbers() {
        let written = serde_json::to_string(&Pan(3.5, -2.0)).expect("it writes");

        assert_eq!(written, "[3.5,-2.0]");
    }

    #[test]
    fn a_pan_written_by_the_old_shim_still_reads() {
        let read: Pan = serde_json::from_str("[3.5,-2.0]").expect("it reads");

        assert_eq!(read, Pan(3.5, -2.0));
    }

    #[test]
    fn a_place_round_trips_as_it_always_did() {
        let place = Place {
            zoom: 2.5,
            pan: Pan(10.0, -4.0),
        };

        let written = serde_json::to_string(&place).expect("it writes");
        assert_eq!(written, r#"{"zoom":2.5,"pan":[10.0,-4.0]}"#);

        let read: Place = serde_json::from_str(&written).expect("it reads");
        assert_eq!(read, place);
    }

    /// The pan is meaningless at a zoom that shows the whole photograph, and
    /// is clamped away on the next frame anyway.
    #[test]
    fn an_untouched_place_is_not_worth_a_map_entry() {
        assert!(!Place::UNTOUCHED.is_worth_remembering());
        assert!(Place {
            zoom: 2.0,
            pan: Pan::NONE
        }
        .is_worth_remembering());
    }
}

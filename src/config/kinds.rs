//! The closed sets of words the configuration file stores.
//!
//! A setting whose value is one of a handful of words — where the overlay sits,
//! how a photograph opens — is a choice the *file* holds, and the drawing that
//! acts on it is somewhere else entirely. These two lived in
//! `view::image_view::`, which meant `Config` named a type in the drawing
//! layer; and through that one edge `organize` and `annotations`, which have
//! never mentioned the toolkit, depended on it transitively.
//!
//! What stayed behind is what genuinely draws: the overlay's painter and the
//! arithmetic that turns an `Opening` into a size.

/// Where on the photograph the overlay sits.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    /// Not drawn at all.
    #[default]
    Off,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub const ALL: &'static [Corner] = &[
        Corner::Off,
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];

    /// The word the file holds for this corner.
    ///
    /// The registry is keyed on what a forum answer quotes, which is what the
    /// document says rather than what the control says.
    pub fn value(self) -> &'static str {
        match self {
            Corner::Off => "off",
            Corner::TopLeft => "top_left",
            Corner::TopRight => "top_right",
            Corner::BottomLeft => "bottom_left",
            Corner::BottomRight => "bottom_right",
        }
    }

    /// The corner that word names, if it names one.
    pub fn of(value: &str) -> Option<Corner> {
        Corner::ALL
            .iter()
            .copied()
            .find(|corner| corner.value() == value)
    }

    /// The next corner round, for the key that cycles it.
    ///
    /// Through the corners and then off, so one key both moves it out of the
    /// way of whatever it is covering and turns it off entirely.
    pub fn next(self) -> Corner {
        match self {
            Corner::Off => Corner::TopLeft,
            Corner::TopLeft => Corner::TopRight,
            Corner::TopRight => Corner::BottomRight,
            Corner::BottomRight => Corner::BottomLeft,
            Corner::BottomLeft => Corner::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Corner::Off => "Off",
            Corner::TopLeft => "Top left",
            Corner::TopRight => "Top right",
            Corner::BottomLeft => "Bottom left",
            Corner::BottomRight => "Bottom right",
        }
    }
}

/// What a newly shown photograph is drawn at.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Opening {
    /// The whole of it, fitted to the panel.
    #[default]
    Fit,
    /// Filling the panel, cropping whichever side is longer.
    Fill,
    /// Exactly as wide as the panel, however tall that leaves it.
    Width,
    /// Exactly as tall as the panel.
    Height,
    /// At a percentage of the photograph's own pixels, which the sibling
    /// setting holds.
    Percent,
}

impl Opening {
    pub const ALL: &'static [Opening] = &[
        Opening::Fit,
        Opening::Fill,
        Opening::Width,
        Opening::Height,
        Opening::Percent,
    ];

    /// The word the file holds.
    ///
    /// The registry is keyed on what a forum answer quotes, which is what the
    /// document says rather than what the control says.
    pub fn value(self) -> &'static str {
        match self {
            Opening::Fit => "fit",
            Opening::Fill => "fill",
            Opening::Width => "width",
            Opening::Height => "height",
            Opening::Percent => "percent",
        }
    }

    /// The opening that word names, if it names one.
    pub fn of(value: &str) -> Option<Opening> {
        Opening::ALL
            .iter()
            .copied()
            .find(|opening| opening.value() == value)
    }

    pub fn label(self) -> &'static str {
        match self {
            Opening::Fit => "Fitted to the window",
            Opening::Fill => "Filling the window",
            Opening::Width => "As wide as the window",
            Opening::Height => "As tall as the window",
            Opening::Percent => "At a magnification you choose",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Opening::Fit => "The whole photograph, as large as the window will take it.",
            Opening::Fill => {
                "Covers the window, cropping whichever side is longer. Nothing is lost \
                 — the rest is a pan away."
            }
            Opening::Width => {
                "As wide as the window, with the top and the bottom cropped if the \
                 photograph is taller than it is wide."
            }
            Opening::Height => {
                "As tall as the window, with the sides cropped. What a folder of \
                 panoramas wants."
            }
            Opening::Percent => {
                "At the magnification below, against the photograph's own pixels: a \
                 hundred per cent is one screen pixel to one of theirs, which is what \
                 focus is judged at."
            }
        }
    }

    /// Whether the magnification beside it is the one this reads.
    ///
    /// The number is drawn and searched for whatever the choice is — a setting
    /// that vanishes is a setting nobody finds — but it is worth saying when
    /// it is doing nothing.
    pub fn reads_the_percentage(self) -> bool {
        self == Opening::Percent
    }

    /// The next one round, for the key that cycles them.
    pub fn next(self) -> Opening {
        let at = Opening::ALL
            .iter()
            .position(|opening| *opening == self)
            .unwrap_or(0);

        Opening::ALL[(at + 1) % Opening::ALL.len()]
    }
}

crate::choices! {
    /// What the collection is ordered by.
    pub enum SortBy {
        #[default]
        Name = "name", "Name", "The order the crawler found them in, which is already natural by name.";
        Stars = "stars", "Stars";
        Label = "label", "Colour label";
        Flag = "flag", "Flag";
    }
}

crate::choices! {
    /// Which flags a photograph may carry to be shown.
    ///
    /// These five had two sets of words: `FlagRule::label` said "Any flag",
    /// "Not rejected" and "Kept", while the registry's table said
    /// "Everything", "Everything but the rejects" and "Only the keepers" —
    /// so the same five choices read differently depending on which window
    /// they were opened from. One table now, with the terse word as the label
    /// the filter bar's chip needs and the longer one as the sentence the
    /// settings window already draws under it.
    pub enum FlagRule {
        #[default]
        Any = "any", "Any flag", "Everything, however it is flagged.";
        NotRejected = "not_rejected", "Not rejected", "Everything but the rejects. The one people leave on during a first pass.";
        Picked = "picked", "Kept", "Only the keepers.";
        Rejected = "rejected", "Rejected", "Only the rejects.";
        Unflagged = "unflagged", "Unflagged", "What is left to decide about.";
    }
}

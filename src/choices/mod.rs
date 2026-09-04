//! A closed set of words, written once.
//!
//! A setting whose value is one of a handful of words is written out three
//! times in this program: as an enum with its `value`, `label` and `ALL`; as a
//! `Choice` table in the registry so the settings window can draw a control for
//! it; and a third time as doc comments on the variants. Twenty-two tables and
//! about seventy literals, with nothing checking that the copies agree.
//!
//! They had already stopped agreeing. The flag filter's five choices carried
//! two different sets of words depending on which window they were opened
//! from, because the enum's `label` and the registry's `Choice::label` are two
//! places to write the same thing and somebody changed one.
//!
//! [`choices!`] declares the enum and derives all of it: the words the file
//! holds, the words a control shows, the sentence under each, the round trip
//! through `serde`, and the walk the cycling key uses. The registry's table is
//! then [`Choices::ROWS`] rather than a second copy.
//!
//! # Why a macro and not a derive
//!
//! A derive would need its own crate — a proc-macro crate cannot live in the
//! crate that uses it — and this is one file. The macro is declarative, so
//! there is no build step, and it expands to plain items a reader can see by
//! expanding it. What it cannot do is be applied to an enum declared
//! elsewhere, which is the whole point: the enum is *declared here* or it is
//! not one of these.

/// One of the words, as everything that draws a control needs it.
///
/// The same three fields the registry's hand-written tables carried, so the
/// settings window did not change at all when the tables stopped being
/// hand-written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice {
    /// What the file holds, which is what a forum answer quotes.
    pub value: &'static str,
    /// What the control says.
    pub label: &'static str,
    /// The line under it. Fewer than five choices get a sentence each, which
    /// is the shape the slideshow's `Motion` control already uses and the best
    /// control in the program.
    pub sentence: &'static str,
}

/// A closed set of words a setting can hold.
///
/// Every implementor is declared by [`choices!`], so the three answers cannot
/// disagree: they are the same table read three ways.
pub trait Choices: Copy + PartialEq + Sized + 'static {
    /// Every one of them, in the order a control lists them.
    const EVERY: &'static [Self];

    /// The same set, as the settings window draws it.
    const ROWS: &'static [Choice];

    /// What the file holds.
    fn value(self) -> &'static str;

    /// What a control calls it.
    fn label(self) -> &'static str;

    /// The line under it in a control, empty where there is none.
    fn sentence(self) -> &'static str;

    /// Reads the word the file holds.
    ///
    /// `None` for a word this build does not know, which the caller answers
    /// with the default — a configuration written by a later version should
    /// cost one setting, not the file.
    fn of(value: &str) -> Option<Self> {
        Self::EVERY.iter().copied().find(|it| it.value() == value)
    }

    /// The next one round, for the key that cycles.
    fn next(self) -> Self {
        let at = Self::EVERY.iter().position(|it| *it == self).unwrap_or(0);

        Self::EVERY[(at + 1) % Self::EVERY.len()]
    }

    /// The one before, for the key that cycles backwards.
    fn previous(self) -> Self {
        let at = Self::EVERY.iter().position(|it| *it == self).unwrap_or(0);

        Self::EVERY[(at + Self::EVERY.len() - 1) % Self::EVERY.len()]
    }
}

/// Declares a closed set of words and everything that reads it.
///
/// ```
/// use avis_imgv::choices;
/// use avis_imgv::choices::Choices;
///
/// choices! {
///     /// How the folder is ordered.
///     pub enum SortBy {
///         #[default]
///         Name = "name", "Name", "The order the crawler found them in.";
///         Stars = "stars", "Stars";
///     }
/// }
///
/// assert_eq!(SortBy::default(), SortBy::Name);
/// assert_eq!(SortBy::Name.value(), "name");
/// assert_eq!(SortBy::of("stars"), Some(SortBy::Stars));
/// assert_eq!(SortBy::Stars.next(), SortBy::Name);
/// assert_eq!(SortBy::ROWS.len(), 2);
/// ```
#[macro_export]
macro_rules! choices {
    (
        $(#[$outer:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$attr:meta])*
                $variant:ident = $value:literal, $label:literal $(, $sentence:literal)? ;
            )+
        }
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        $vis enum $name {
            $(
                $(#[$attr])*
                #[doc = $label]
                $variant,
            )+
        }

        impl $crate::choices::Choices for $name {
            const EVERY: &'static [Self] = &[$(Self::$variant),+];

            const ROWS: &'static [$crate::choices::Choice] = &[
                $($crate::choices::Choice {
                    value: $value,
                    label: $label,
                    sentence: $crate::choices!(@sentence $($sentence)?),
                }),+
            ];

            fn value(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }

            fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label),+
                }
            }

            fn sentence(self) -> &'static str {
                match self {
                    $(Self::$variant => $crate::choices!(@sentence $($sentence)?)),+
                }
            }
        }

        // The file's spelling, both ways, so a set cannot round-trip through
        // `serde` as something else. An unknown word is the default rather
        // than an error: a configuration written by a later version should
        // cost one setting, not the file.
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                use $crate::choices::Choices;

                let word = <std::string::String as serde::Deserialize>::deserialize(d)?;

                Ok(<Self as Choices>::of(&word).unwrap_or_default())
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                use $crate::choices::Choices;

                s.serialize_str(self.value())
            }
        }

        // Every word is spelt once, every label is used once, and there is at
        // least one of them. A real `E0080` at build time rather than a test
        // somebody has to remember to write.
        const _: () = {
            let rows = <$name as $crate::choices::Choices>::ROWS;
            assert!(!rows.is_empty(), "a closed set with nothing in it");

            let mut i = 0;
            while i < rows.len() {
                assert!(
                    !rows[i].value.is_empty(),
                    "a choice with no word for the file to hold",
                );
                assert!(!rows[i].label.is_empty(), "a choice with nothing to call it");

                let mut j = i + 1;
                while j < rows.len() {
                    assert!(
                        !$crate::choices::same(rows[i].value, rows[j].value),
                        "two choices spelt the same way in the file",
                    );
                    assert!(
                        !$crate::choices::same(rows[i].label, rows[j].label),
                        "two choices a control would draw identically",
                    );
                    j += 1;
                }

                i += 1;
            }
        };
    };

    (@sentence) => { "" };
    (@sentence $sentence:literal) => { $sentence };
}

/// Whether two words are the same, in a `const`.
///
/// `str::eq` is not usable in a constant, and the check that no two choices
/// share a word has to run at build time or it is a test somebody can forget.
pub const fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());

    if a.len() != b.len() {
        return false;
    }

    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    choices! {
        /// A set to test the machinery with.
        pub enum Season {
            #[default]
            Spring = "spring", "Spring", "When the light is best.";
            Summer = "summer", "Summer";
            Autumn = "autumn", "Autumn";
        }
    }

    #[test]
    fn the_default_is_the_variant_that_says_so() {
        assert_eq!(Season::default(), Season::Spring);
    }

    #[test]
    fn every_variant_is_in_the_list_once() {
        assert_eq!(Season::EVERY.len(), 3);
        assert_eq!(Season::ROWS.len(), 3);
    }

    /// The point of the whole thing: the words a control draws and the words
    /// the file holds are the same table, so they cannot drift.
    #[test]
    fn the_rows_and_the_variants_are_one_table() {
        for (row, variant) in Season::ROWS.iter().zip(Season::EVERY) {
            assert_eq!(row.value, variant.value());
            assert_eq!(row.label, variant.label());
            assert_eq!(row.sentence, variant.sentence());
        }
    }

    #[test]
    fn a_word_the_file_holds_reads_back() {
        assert_eq!(Season::of("autumn"), Some(Season::Autumn));
        assert_eq!(Season::of("Autumn"), None, "the file's spelling, exactly");
        assert_eq!(Season::of("winter"), None);
    }

    #[test]
    fn a_set_with_no_sentence_has_an_empty_one() {
        assert_eq!(Season::Spring.sentence(), "When the light is best.");
        assert_eq!(Season::Summer.sentence(), "");
    }

    #[test]
    fn cycling_goes_round() {
        assert_eq!(Season::Spring.next(), Season::Summer);
        assert_eq!(Season::Autumn.next(), Season::Spring);
        assert_eq!(Season::Spring.previous(), Season::Autumn);
    }

    /// A configuration written by a later version should cost one setting,
    /// not the file.
    #[test]
    fn a_word_this_build_does_not_know_reads_as_the_default() {
        let read: Season = serde_json::from_str("\"winter\"").expect("it still parses");

        assert_eq!(read, Season::Spring);
    }

    #[test]
    fn a_set_round_trips_through_the_file() {
        for season in Season::EVERY {
            let written = serde_json::to_string(season).expect("it writes");
            let read: Season = serde_json::from_str(&written).expect("it reads");

            assert_eq!(read, *season);
            assert_eq!(written, format!("\"{}\"", season.value()));
        }
    }

    #[test]
    fn words_are_compared_in_a_constant() {
        const _: () = assert!(same("spring", "spring"));
        const _: () = assert!(!same("spring", "summer"));
        const _: () = assert!(!same("spring", "spr"));

        assert!(same("", ""));
    }
}

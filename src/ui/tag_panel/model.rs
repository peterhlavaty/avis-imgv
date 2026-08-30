//! What the tagging panel should show, worked out apart from how it looks.

use crate::annotations::catalog::Group;
use crate::annotations::{Catalog, RecentTags};
use crate::metadata::xmp::Xmp;

/// Everything the panel draws from.
pub struct Source<'a> {
    /// Annotations of the image on screen.
    pub annotations: &'a Xmp,
    /// Tags the user configured, grouped into categories.
    pub catalog: &'a Catalog,
    /// Tags applied recently, most recent first.
    pub recent: &'a RecentTags,
    /// Tags seen on the other images of this folder, so a tag typed once is
    /// offered again without having to be configured.
    pub seen: &'a [&'a str],
}

/// The panel's contents once the search box has been applied.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sections {
    /// Tags already on the image, in the order they are stored.
    pub on_image: Vec<String>,
    /// Recently used tags that are not already on the image.
    pub recent: Vec<String>,
    /// Configured tags, keeping their categories.
    pub categories: Vec<Group>,
    /// Tags seen elsewhere in the folder but neither recent nor configured.
    pub seen: Vec<String>,
    /// The search text, when it names a tag that exists nowhere yet and could
    /// be created.
    pub create: Option<String>,
}

impl Sections {
    /// True when the search matched nothing at all.
    pub fn is_empty(&self) -> bool {
        self.recent.is_empty() && self.categories.is_empty() && self.seen.is_empty()
    }
}

/// Works out what to offer for `query`.
///
/// Tags on the image are always listed in full: they are what the panel is
/// there to show, and hiding them behind a search would be confusing. Every
/// other section is filtered, and a tag is only offered once — the nearest
/// section to hand wins.
pub fn sections(source: &Source<'_>, query: &str) -> Sections {
    let query = query.trim();
    let needle = query.to_lowercase();

    let on_image = source.annotations.keywords.clone();
    let has = |tag: &str| on_image.iter().any(|existing| existing == tag);

    let recent: Vec<String> = source
        .recent
        .tags()
        .filter(|tag| !has(tag) && matches(tag, &needle))
        .map(str::to_string)
        .collect();

    let categories: Vec<Group> = source
        .catalog
        .search(query)
        .into_iter()
        .filter_map(|group| {
            let tags: Vec<String> = group
                .tags
                .into_iter()
                .filter(|tag| !has(tag) && !recent.contains(tag))
                .collect();

            (!tags.is_empty()).then_some(Group {
                category: group.category,
                tags,
            })
        })
        .collect();

    let configured = source.catalog.tags();
    let seen: Vec<String> = source
        .seen
        .iter()
        .filter(|tag| {
            !has(tag)
                && !recent.iter().any(|recent| recent == *tag)
                && !configured.contains(*tag)
                && matches(tag, &needle)
        })
        .map(|tag| tag.to_string())
        .collect();

    let known = |tag: &str| tag.eq_ignore_ascii_case(query);
    let exists = on_image.iter().any(|tag| known(tag))
        || recent.iter().any(|tag| known(tag))
        || seen.iter().any(|tag| known(tag))
        || configured.iter().any(|tag| known(tag));

    Sections {
        create: (!query.is_empty() && !exists).then(|| query.to_string()),
        on_image,
        recent,
        categories,
        seen,
    }
}

/// Case insensitive substring match; an empty needle matches everything.
fn matches(tag: &str, lowercase_needle: &str) -> bool {
    lowercase_needle.is_empty() || tag.to_lowercase().contains(lowercase_needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TagCategory;

    fn catalog() -> Catalog {
        Catalog::new(vec![
            TagCategory {
                name: "Places".to_string(),
                tags: vec!["Slovakia".to_string(), "Tatras".to_string()],
            },
            TagCategory {
                name: "Subject".to_string(),
                tags: vec!["Macro".to_string()],
            },
        ])
    }

    fn recent(tags: &[&str]) -> RecentTags {
        let mut recent = RecentTags::with_limit(8);
        for tag in tags.iter().rev() {
            recent.remember(*tag);
        }

        recent
    }

    fn annotations(keywords: &[&str]) -> Xmp {
        Xmp {
            rating: 0,
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
            ..Xmp::default()
        }
    }

    #[test]
    fn an_empty_query_offers_everything() {
        let annotations = annotations(&["Autumn"]);
        let catalog = catalog();
        let recent = recent(&["Tatras"]);
        let source = Source {
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            seen: &["Fog"],
        };

        let sections = sections(&source, "");

        assert_eq!(sections.on_image, vec!["Autumn"]);
        assert_eq!(sections.recent, vec!["Tatras"]);
        assert_eq!(sections.seen, vec!["Fog"]);
        assert_eq!(sections.categories.len(), 2);
        assert_eq!(sections.create, None);
    }

    #[test]
    fn a_tag_is_offered_once_and_by_the_nearest_section() {
        let annotations = annotations(&["Macro"]);
        let catalog = catalog();
        let recent = recent(&["Tatras"]);
        let source = Source {
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            // Configured and recent already, so not offered again.
            seen: &["Tatras", "Slovakia"],
        };

        let sections = sections(&source, "");

        // Macro is on the image, so its category drops out entirely.
        assert!(sections
            .categories
            .iter()
            .all(|group| group.category != "Subject"));
        // Tatras is recent, so Places offers only Slovakia.
        assert_eq!(sections.categories[0].tags, vec!["Slovakia"]);
        assert!(sections.seen.is_empty());
    }

    #[test]
    fn tags_on_the_image_are_never_filtered_away() {
        let annotations = annotations(&["Autumn"]);
        let catalog = catalog();
        let recent = recent(&[]);
        let source = Source {
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            seen: &[],
        };

        assert_eq!(sections(&source, "zzz").on_image, vec!["Autumn"]);
    }

    #[test]
    fn searching_a_category_offers_all_of_it() {
        let annotations = annotations(&[]);
        let catalog = catalog();
        let recent = recent(&[]);
        let source = Source {
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            seen: &[],
        };

        let sections = sections(&source, "places");

        assert_eq!(sections.categories.len(), 1);
        assert_eq!(sections.categories[0].tags, vec!["Slovakia", "Tatras"]);
    }

    #[test]
    fn an_unknown_query_can_be_created() {
        let annotations = annotations(&[]);
        let catalog = catalog();
        let recent = recent(&[]);
        let source = Source {
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            seen: &[],
        };

        let sections = sections(&source, "Underwater");

        assert_eq!(sections.create.as_deref(), Some("Underwater"));
        assert!(sections.is_empty());
    }

    #[test]
    fn a_query_naming_an_existing_tag_is_not_offered_for_creation() {
        let annotations = annotations(&["Autumn"]);
        let catalog = catalog();
        let recent = recent(&[]);
        let source = Source {
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            seen: &[],
        };

        // Whatever case it is typed in, and wherever it already lives.
        assert_eq!(sections(&source, "autumn").create, None);
        assert_eq!(sections(&source, "TATRAS").create, None);
    }

    #[test]
    fn a_blank_query_creates_nothing() {
        let annotations = annotations(&[]);
        let catalog = catalog();
        let recent = recent(&[]);
        let source = Source {
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            seen: &[],
        };

        assert_eq!(sections(&source, "   ").create, None);
    }
}

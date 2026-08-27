//! The tags a user keeps to hand, grouped into categories, and searching them.

use std::collections::BTreeSet;

use crate::config::TagCategory;

/// A tag offered by the panel, and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub tag: String,
    /// The category the tag is filed under, or `None` for one that only exists
    /// on an image.
    pub category: Option<String>,
}

/// One category's worth of matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub category: String,
    pub tags: Vec<String>,
}

/// The configured tags, ready to be searched.
#[derive(Debug, Default, Clone)]
pub struct Catalog {
    categories: Vec<TagCategory>,
}

impl Catalog {
    pub fn new(categories: Vec<TagCategory>) -> Catalog {
        Catalog { categories }
    }

    pub fn is_empty(&self) -> bool {
        self.categories.iter().all(|group| group.tags.is_empty())
    }

    /// Every tag in the catalog, deduplicated and sorted.
    pub fn tags(&self) -> BTreeSet<&str> {
        self.categories
            .iter()
            .flat_map(|group| group.tags.iter().map(String::as_str))
            .collect()
    }

    /// The categories a tag belongs to. A tag may be filed in several.
    pub fn categories_of(&self, tag: &str) -> Vec<&str> {
        self.categories
            .iter()
            .filter(|group| group.tags.iter().any(|candidate| candidate == tag))
            .map(|group| group.name.as_str())
            .collect()
    }

    /// The catalog filtered by `query`, keeping the configured grouping.
    ///
    /// A query matches a tag by its own name or by the name of the category it
    /// is filed under, so searching for "places" offers everything in that
    /// category.
    pub fn search(&self, query: &str) -> Vec<Group> {
        let query = query.trim().to_lowercase();

        self.categories
            .iter()
            .filter_map(|group| {
                let category_matches = contains(&group.name, &query);

                let tags: Vec<String> = group
                    .tags
                    .iter()
                    .filter(|tag| category_matches || contains(tag, &query))
                    .cloned()
                    .collect();

                (!tags.is_empty()).then(|| Group {
                    category: group.name.clone(),
                    tags,
                })
            })
            .collect()
    }

    /// A flat list of matches, for offering completions.
    pub fn suggest(&self, query: &str) -> Vec<Suggestion> {
        self.search(query)
            .into_iter()
            .flat_map(|group| {
                let category = group.category;
                group.tags.into_iter().map(move |tag| Suggestion {
                    tag,
                    category: Some(category.clone()),
                })
            })
            .collect()
    }
}

/// Case insensitive substring match. An empty query matches everything.
fn contains(haystack: &str, lowercase_query: &str) -> bool {
    lowercase_query.is_empty() || haystack.to_lowercase().contains(lowercase_query)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Catalog {
        Catalog::new(vec![
            TagCategory {
                name: "Places".to_string(),
                tags: vec!["Slovakia".to_string(), "Tatras".to_string()],
            },
            TagCategory {
                name: "Subjects".to_string(),
                tags: vec!["Portrait".to_string(), "Macro".to_string()],
            },
            TagCategory {
                name: "Status".to_string(),
                tags: vec!["Portfolio".to_string()],
            },
        ])
    }

    #[test]
    fn an_empty_query_offers_everything() {
        let groups = catalog().search("");

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].tags, vec!["Slovakia", "Tatras"]);
    }

    #[test]
    fn a_query_matches_tag_names() {
        let groups = catalog().search("portra");

        assert_eq!(
            groups,
            vec![Group {
                category: "Subjects".to_string(),
                tags: vec!["Portrait".to_string()],
            }]
        );
    }

    #[test]
    fn a_query_reaches_across_categories() {
        // "port" is in both Portrait and Portfolio.
        let groups = catalog().search("port");

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].tags, vec!["Portrait"]);
        assert_eq!(groups[1].tags, vec!["Portfolio"]);
    }

    #[test]
    fn a_query_matching_a_category_offers_all_of_it() {
        let groups = catalog().search("places");

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].category, "Places");
        assert_eq!(groups[0].tags, vec!["Slovakia", "Tatras"]);
    }

    #[test]
    fn searching_ignores_case_and_surrounding_space() {
        assert_eq!(catalog().search("  MACRO ").len(), 1);
        assert_eq!(catalog().search("macro")[0].tags, vec!["Macro"]);
    }

    #[test]
    fn a_query_that_matches_nothing_returns_nothing() {
        assert!(catalog().search("underwater").is_empty());
    }

    #[test]
    fn a_tag_reports_the_categories_it_is_filed_under() {
        let catalog = Catalog::new(vec![
            TagCategory {
                name: "Places".to_string(),
                tags: vec!["Tatras".to_string()],
            },
            TagCategory {
                name: "Favourites".to_string(),
                tags: vec!["Tatras".to_string()],
            },
        ]);

        assert_eq!(
            catalog.categories_of("Tatras"),
            vec!["Places", "Favourites"]
        );
        assert!(catalog.categories_of("Nowhere").is_empty());
    }

    #[test]
    fn suggestions_are_flat_and_carry_their_category() {
        let suggestions = catalog().suggest("tatras");

        assert_eq!(
            suggestions,
            vec![Suggestion {
                tag: "Tatras".to_string(),
                category: Some("Places".to_string()),
            }]
        );
    }

    #[test]
    fn an_unconfigured_catalog_is_empty() {
        assert!(Catalog::default().is_empty());
        assert!(Catalog::default().search("anything").is_empty());
        assert!(Catalog::new(vec![TagCategory {
            name: "Empty".to_string(),
            tags: vec![],
        }])
        .is_empty());
    }

    #[test]
    fn every_tag_is_listed_once() {
        let catalog = Catalog::new(vec![
            TagCategory {
                name: "A".to_string(),
                tags: vec!["Shared".to_string(), "One".to_string()],
            },
            TagCategory {
                name: "B".to_string(),
                tags: vec!["Shared".to_string()],
            },
        ]);

        assert_eq!(
            catalog.tags().into_iter().collect::<Vec<_>>(),
            vec!["One", "Shared"]
        );
    }
}

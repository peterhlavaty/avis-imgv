//! The tags a user keeps to hand, grouped into categories, and searching them.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::config::{Config, TagCategory, TagConfig};
use crate::metadata::xmp::{leaf_of, HIERARCHY_SEPARATOR};

/// A line beginning with this is a note to the person editing the file.
const COMMENT: char = '#';

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

    /// The catalog the configuration asks for: what is written in the file,
    /// and then whatever keyword list it points at.
    ///
    /// A list that cannot be read is a warning in the log rather than a
    /// failure to start: the viewer is still a viewer without it, and somebody
    /// who has just renamed a file should not be locked out of their
    /// photographs over it.
    pub fn configured(config: &TagConfig) -> Catalog {
        let mut catalog = Catalog::new(config.categories.clone());

        let Some(named) = &config.catalog_file else {
            return catalog;
        };

        let path = beside_the_config(named);
        match read_file(&path) {
            Ok(categories) => catalog.merge(categories),
            Err(error) => {
                tracing::warn!(
                    "could not read the keyword list {}: {error}",
                    path.display()
                )
            }
        }

        catalog
    }

    /// Adds categories to the catalog, folding those that share a name.
    fn merge(&mut self, categories: Vec<TagCategory>) {
        for group in categories {
            match self
                .categories
                .iter_mut()
                .find(|existing| existing.name == group.name)
            {
                Some(existing) => {
                    for tag in group.tags {
                        if !existing.tags.iter().any(|known| same_tag(known, &tag)) {
                            existing.tags.push(tag);
                        }
                    }
                }
                None => self.categories.push(group),
            }
        }
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

/// Reads a keyword list from a text file.
///
/// Every photo application worth the name can export its keywords as an
/// indented list, and a photographer who has spent years building one in
/// Lightroom or digiKam should not have to type it again into a JSON file. The
/// format is theirs:
///
/// ```text
/// Places
///     Slovakia
///         Tatras
///     Austria
/// Subjects
///     Portrait
/// ```
///
/// Indentation makes the hierarchy — tabs or spaces, as long as a deeper level
/// is indented further than its parent. The outermost level becomes a category
/// in the panel and every tag below it keeps its whole path, so clicking
/// `Tatras` writes `Places|Slovakia|Tatras` into the sidecar as well as the
/// plain keyword.
///
/// A line that already holds bars is taken as a path as it stands, so a flat
/// file of `Places|Slovakia|Tatras` lines reads as well as an indented one.
/// Blank lines and lines starting with `#` are ignored.
pub fn read_file(path: &Path) -> std::io::Result<Vec<TagCategory>> {
    Ok(from_text(&std::fs::read_to_string(path)?))
}

/// Where a keyword list named in the configuration actually lives.
///
/// A relative name is taken as relative to the configuration file, which is
/// where somebody writing `"keywords.txt"` into it means.
fn beside_the_config(named: &str) -> PathBuf {
    let named = PathBuf::from(named);
    if named.is_absolute() {
        return named;
    }

    match Config::path().and_then(|path| path.parent().map(Path::to_path_buf)) {
        Some(directory) => directory.join(named),
        None => named,
    }
}

/// The parsing half of [`read_file`], apart from the reading so it can be
/// tested without a file.
pub fn from_text(text: &str) -> Vec<TagCategory> {
    let mut categories: Vec<TagCategory> = Vec::new();
    // The path down to the line last read, one entry per level of indentation,
    // holding how far that level is indented and what it is called.
    let mut ancestry: Vec<(usize, String)> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(COMMENT) {
            continue;
        }

        let indent = line.len() - line.trim_start().len();

        // Back out to the level this line belongs to. Equal indentation is a
        // sibling, so it pops as well.
        while ancestry.last().is_some_and(|(depth, _)| *depth >= indent) {
            ancestry.pop();
        }

        // A line with bars in it carries its own ancestry and starts afresh.
        let levels: Vec<&str> = crate::metadata::xmp::levels_of(trimmed);
        if levels.is_empty() {
            continue;
        }

        if levels.len() > 1 {
            ancestry.clear();
        }

        for (nth, level) in levels.iter().enumerate() {
            let depth = indent + nth;
            ancestry.push((depth, (*level).to_string()));
        }

        let path: Vec<&str> = ancestry.iter().map(|(_, name)| name.as_str()).collect();

        // The outermost level names the category; it is a heading rather than a
        // keyword of its own, which is how these files are written.
        let Some((category, below)) = path.split_first() else {
            continue;
        };

        if below.is_empty() {
            if !categories.iter().any(|group| group.name == *category) {
                categories.push(TagCategory {
                    name: (*category).to_string(),
                    tags: Vec::new(),
                });
            }

            continue;
        }

        let tag = path.join(&HIERARCHY_SEPARATOR.to_string());
        match categories.iter_mut().find(|group| group.name == *category) {
            Some(group) => {
                if !group.tags.contains(&tag) {
                    group.tags.push(tag);
                }
            }
            None => categories.push(TagCategory {
                name: (*category).to_string(),
                tags: vec![tag],
            }),
        }
    }

    categories
}

/// Whether two tags name the same keyword.
///
/// `Places|Slovakia|Tatras` and `Tatras` are one keyword filed two ways, and a
/// panel that offered both would be offering the user a choice that does not
/// exist.
pub fn same_tag(one: &str, other: &str) -> bool {
    leaf_of(one) == leaf_of(other)
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

    /// The shape of a keyword list exported from Lightroom or digiKam.
    const KEYWORD_FILE: &str = "# The places I shoot
Places
	Slovakia
		Tatras
		Bratislava
	Austria

Subjects
	Portrait
";

    #[test]
    fn a_keyword_file_keeps_the_whole_path_of_every_tag() {
        let categories = from_text(KEYWORD_FILE);

        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].name, "Places");
        assert_eq!(
            categories[0].tags,
            vec![
                "Places|Slovakia",
                "Places|Slovakia|Tatras",
                "Places|Slovakia|Bratislava",
                "Places|Austria",
            ]
        );
        assert_eq!(categories[1].tags, vec!["Subjects|Portrait"]);
    }

    #[test]
    fn a_flat_file_of_paths_reads_the_same_as_an_indented_one() {
        let flat = from_text(
            "Places|Slovakia
Places|Slovakia|Tatras
Places|Slovakia|Bratislava
Places|Austria
Subjects|Portrait
",
        );

        assert_eq!(flat, from_text(KEYWORD_FILE));
    }

    #[test]
    fn spaces_indent_as_well_as_tabs() {
        let categories = from_text(
            "Places
    Slovakia
        Tatras
",
        );

        assert_eq!(
            categories[0].tags,
            vec!["Places|Slovakia", "Places|Slovakia|Tatras"]
        );
    }

    /// A heading with nothing under it is still a category, so somebody
    /// building a list top down sees it appear as they write it.
    #[test]
    fn a_category_with_no_tags_survives() {
        let categories = from_text(
            "Places
Subjects
	Portrait
",
        );

        assert_eq!(categories.len(), 2);
        assert!(categories[0].tags.is_empty());
    }

    #[test]
    fn the_same_tag_written_twice_is_kept_once() {
        let categories = from_text(
            "Places
	Tatras
Places
	Tatras
",
        );

        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].tags, vec!["Places|Tatras"]);
    }

    #[test]
    fn a_tag_is_the_same_tag_however_it_is_filed() {
        assert!(same_tag("Places|Slovakia|Tatras", "Tatras"));
        assert!(same_tag("Places|Tatras", "Subjects|Tatras"));
        assert!(!same_tag("Places|Slovakia", "Places|Austria"));
    }

    #[test]
    fn a_keyword_list_read_from_a_file_joins_the_configured_categories() {
        let dir = std::env::temp_dir().join("avis-catalog-file");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("keywords.txt");
        std::fs::write(&file, KEYWORD_FILE).unwrap();

        let config = TagConfig {
            categories: vec![TagCategory {
                name: "Places".to_string(),
                tags: vec!["Places|Slovakia".to_string()],
            }],
            catalog_file: Some(file.display().to_string()),
            ..TagConfig::default()
        };

        let catalog = Catalog::configured(&config);
        let places = catalog.search("Places").remove(0);

        // The category is folded rather than repeated, and what was already
        // configured is not listed twice.
        assert_eq!(catalog.categories_of("Subjects|Portrait"), vec!["Subjects"]);
        assert_eq!(
            places.tags,
            vec![
                "Places|Slovakia",
                "Places|Slovakia|Tatras",
                "Places|Slovakia|Bratislava",
                "Places|Austria",
            ]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A list that is not there is a warning, not a failure to start.
    #[test]
    fn a_missing_keyword_list_leaves_the_configured_tags_alone() {
        let config = TagConfig {
            categories: vec![TagCategory {
                name: "Places".to_string(),
                tags: vec!["Tatras".to_string()],
            }],
            catalog_file: Some(
                std::env::temp_dir()
                    .join("avis-no-such-keywords.txt")
                    .display()
                    .to_string(),
            ),
            ..TagConfig::default()
        };

        assert_eq!(
            Catalog::configured(&config)
                .tags()
                .into_iter()
                .collect::<Vec<_>>(),
            vec!["Tatras"]
        );
    }
}

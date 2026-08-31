//! What the tagging panel should show, worked out apart from how it looks.

use crate::annotations::catalog::{same_tag, Group};
use crate::annotations::{Catalog, RecentTags};
use crate::metadata::xmp::{leaf_of, levels_of, Xmp, HIERARCHY_SEPARATOR};

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
    /// How many photographs what is clicked will be applied to.
    ///
    /// The panel draws one photograph's marks whatever this says, because
    /// there is no sensible way to draw two hundred; it says the number out
    /// loud instead, so nobody clicks a keyword thinking it lands on one.
    pub applies_to: usize,
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

    let on_image = on_image(source.annotations);
    let has = |tag: &str| on_image.iter().any(|existing| same_tag(existing, tag));

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
                .filter(|tag| !has(tag) && !recent.iter().any(|already| same_tag(already, tag)))
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
                && !recent.iter().any(|recent| same_tag(recent, tag))
                && !configured.iter().any(|known| same_tag(known, tag))
                && matches(tag, &needle)
        })
        .map(|tag| tag.to_string())
        .collect();

    let known = |tag: &str| leaf_of(tag).eq_ignore_ascii_case(leaf_of(query));
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

/// The tags on the photograph, each shown with as much of its path as the
/// sidecar records.
///
/// The flat keywords are what every reader understands and what the panel has
/// always drawn, but a photograph tagged `Places|Slovakia|Tatras` in Lightroom
/// should not read as a bare `Tatras` here — the levels are half of what the
/// keyword means. So a keyword that a path ends in is shown as that path.
fn on_image(annotations: &Xmp) -> Vec<String> {
    annotations
        .keywords
        .iter()
        .map(|keyword| {
            annotations
                .hierarchy
                .iter()
                .find(|path| leaf_of(path) == keyword)
                .unwrap_or(keyword)
                .clone()
        })
        .collect()
}

/// One line of a tag tree: how deep it sits, what it is called, and the whole
/// path that clicking it applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub depth: usize,
    pub leaf: String,
    pub path: String,
}

/// Lays a list of tags out as a tree.
///
/// A category of forty keywords filed three levels deep is unreadable as a
/// wrapped row of chips: the same word appears under two parents and there is
/// nothing on screen to say which is which. Drawn as a tree it reads the way it
/// was written.
///
/// Levels that nothing names directly are given a line of their own — a file
/// that lists only `Places|Slovakia|Tatras` still shows Slovakia, because a
/// tree with a hole in it is worse than one with a heading nobody asked for.
pub fn rows(tags: &[String]) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();

    for tag in tags {
        let levels = levels_of(tag);
        let mut path = String::new();

        for (depth, level) in levels.iter().enumerate() {
            if depth > 0 {
                path.push(HIERARCHY_SEPARATOR);
            }
            path.push_str(level);

            if rows.iter().any(|row| row.path == path) {
                continue;
            }

            rows.push(Row {
                depth,
                leaf: (*level).to_string(),
                path: path.clone(),
            });
        }
    }

    rows
}

/// The same, for tags drawn under a heading that already names their outermost
/// level.
///
/// A keyword file's categories are its top level, so `Places` becomes both the
/// heading and the first line of the tree beneath it. One of the two is enough:
/// the heading stays and the row goes, taking a level of indentation with it.
pub fn rows_under(title: &str, tags: &[String]) -> Vec<Row> {
    let rows = rows(tags);
    let redundant = rows
        .iter()
        .filter(|row| row.depth == 0)
        .all(|row| row.leaf == title);

    if !redundant {
        return rows;
    }

    rows.into_iter()
        .filter(|row| row.depth > 0)
        .map(|row| Row {
            depth: row.depth - 1,
            ..row
        })
        .collect()
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
            applies_to: 1,
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
            applies_to: 1,
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
            applies_to: 1,
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
            applies_to: 1,
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
            applies_to: 1,
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
            applies_to: 1,
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
            applies_to: 1,
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            seen: &[],
        };

        assert_eq!(sections(&source, "   ").create, None);
    }

    fn paths(tags: &[&str]) -> Vec<String> {
        tags.iter().map(|tag| (*tag).to_string()).collect()
    }

    #[test]
    fn a_keyword_is_shown_with_the_path_the_sidecar_records() {
        let annotations = Xmp {
            keywords: vec!["Tatras".to_string(), "Winter".to_string()],
            hierarchy: vec!["Places|Slovakia|Tatras".to_string()],
            ..Xmp::default()
        };
        let catalog = catalog();
        let recent = recent(&[]);
        let source = Source {
            applies_to: 1,
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            seen: &[],
        };

        let sections = sections(&source, "");

        // The filed one keeps its levels; the loose one is left alone.
        assert_eq!(sections.on_image, vec!["Places|Slovakia|Tatras", "Winter"]);
    }

    #[test]
    fn a_tag_on_the_image_is_not_offered_again_under_a_path() {
        let annotations = Xmp {
            keywords: vec!["Tatras".to_string()],
            ..Xmp::default()
        };
        let catalog = Catalog::new(vec![TagCategory {
            name: "Places".to_string(),
            tags: paths(&["Places|Slovakia|Tatras", "Places|Austria"]),
        }]);
        let recent = recent(&[]);
        let source = Source {
            applies_to: 1,
            annotations: &annotations,
            catalog: &catalog,
            recent: &recent,
            seen: &["Tatras"],
        };

        let offered = sections(&source, "");

        assert_eq!(offered.categories[0].tags, vec!["Places|Austria"]);
        assert!(offered.seen.is_empty());
        // And typing it out in full offers no new tag to create.
        assert_eq!(sections(&source, "Places|Slovakia|Tatras").create, None);
    }

    #[test]
    fn a_tree_names_every_level_once() {
        let rows = rows(&paths(&[
            "Places|Slovakia|Tatras",
            "Places|Slovakia|Bratislava",
            "Places|Austria",
        ]));

        let shape: Vec<(usize, &str)> = rows
            .iter()
            .map(|row| (row.depth, row.leaf.as_str()))
            .collect();

        assert_eq!(
            shape,
            vec![
                (0, "Places"),
                (1, "Slovakia"),
                (2, "Tatras"),
                (2, "Bratislava"),
                (1, "Austria"),
            ]
        );
    }

    /// A level nothing names directly still gets a line, so the tree has no
    /// holes in it.
    #[test]
    fn a_missing_parent_is_filled_in() {
        let rows = rows(&paths(&["Places|Slovakia|Tatras"]));

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1].path, "Places|Slovakia");
    }

    #[test]
    fn a_row_applies_its_whole_path() {
        let rows = rows(&paths(&["Places|Slovakia|Tatras"]));

        assert_eq!(rows[2].path, "Places|Slovakia|Tatras");
        assert_eq!(rows[2].leaf, "Tatras");
    }

    #[test]
    fn a_heading_that_repeats_the_top_level_takes_it_off_the_tree() {
        let tags = paths(&["Places|Slovakia|Tatras", "Places|Austria"]);
        let rows = rows_under("Places", &tags);

        let shape: Vec<(usize, &str)> = rows
            .iter()
            .map(|row| (row.depth, row.leaf.as_str()))
            .collect();

        assert_eq!(shape, vec![(0, "Slovakia"), (1, "Tatras"), (0, "Austria")]);
        // The path applied is still the whole one.
        assert_eq!(rows[1].path, "Places|Slovakia|Tatras");
    }

    /// Only when the heading really is the one root. A recent list holding
    /// tags from two trees keeps both.
    #[test]
    fn a_heading_over_several_trees_takes_nothing_off() {
        let tags = paths(&["Places|Austria", "Subjects|Portrait"]);

        assert_eq!(rows_under("Recent", &tags), rows(&tags));
    }
}

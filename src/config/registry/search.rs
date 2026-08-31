//! Finding a setting by what somebody would call it.
//!
//! This is the best-evidenced request in the whole of the research behind the
//! plan: five darktable issues by five different people asking for a search box
//! in the preferences, every one of them closed by a stale bot.
//!
//! Four things make it work, and none of them is clever. The label and the
//! sentence are indexed, because the sentence is where the words a person
//! actually uses live. Authored aliases carry other programs' vocabulary and
//! the *complaint* rather than the noun, in their own spelling. An equivalence
//! table folds the spellings nobody should have to guess between. And a stop
//! word list means "where do rejects go" becomes "rejects", which lands.

use super::Row;

/// Words that carry no meaning in a query about a setting.
const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "is", "are", "do", "does", "did", "how", "what", "where", "when", "why",
    "which", "i", "my", "me", "to", "of", "in", "on", "for", "it", "its", "be", "can", "should",
    "would", "set", "setting", "settings", "option", "change", "make", "get", "go", "goes", "and",
    "or", "with", "this", "that",
];

/// Spellings that mean the same thing.
///
/// Each row is one meaning; any word in it matches any other. This is why the
/// authored aliases can be written in another program's spelling — "color
/// class" is Photo Mechanic's, and the table handles the rest.
const SAME: &[&[&str]] = &[
    &["colour", "color"],
    &["grey", "gray"],
    &["memory", "ram"],
    &["gpu", "graphics", "vram", "card", "adapter", "video"],
    &[
        "raw", "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "raf",
    ],
    &["photograph", "photo", "picture", "image", "img"],
    &["folder", "directory", "dir"],
    &["thumbnail", "thumb", "thumbnails"],
    &["keyword", "tag", "keywords", "tags"],
    &["bin", "trash", "recycle"],
    &["delete", "remove", "erase"],
    &["star", "stars", "rating", "rate"],
    &["key", "keys", "shortcut", "shortcuts", "binding", "hotkey"],
    &["mouse", "pointer", "cursor"],
    &["wheel", "scroll", "scrolling"],
    &["size", "resolution", "large", "big"],
    &["slow", "speed", "fast", "performance"],
    &["blurry", "soft", "fuzzy", "blurred"],
    &["window", "screen", "display", "monitor"],
];

/// One result, with how well it matched.
pub struct Hit {
    pub row: &'static Row,
    /// Higher is better. An exact path match wins outright.
    pub score: u32,
    /// Whether every word of the query was found. A failed AND is re-run as an
    /// OR, under a line saying so, because an empty result is the one answer a
    /// search box may never give.
    pub matched_everything: bool,
}

/// Every row worth showing for `query`, best first.
///
/// Never empty while the registry has rows: if nothing matches every word, the
/// rows matching any word come back with `matched_everything` false, and the
/// caller draws them under "Nothing matched all of that. The closest:".
pub fn find(query: &str) -> Vec<Hit> {
    let words = meaningful(query);

    if words.is_empty() {
        return Vec::new();
    }

    // Pasted from a forum post, which is the shape a good answer travels in.
    let exact = query.trim().to_lowercase();
    if let Some(row) = super::rows().iter().find(|row| row.path == exact) {
        return vec![Hit {
            row,
            score: u32::MAX,
            matched_everything: true,
        }];
    }

    let mut all: Vec<Hit> = Vec::new();
    let mut any: Vec<Hit> = Vec::new();

    for row in super::rows() {
        let mut score = 0;
        let mut found_all = true;

        for word in &words {
            match scores(row, word) {
                0 => found_all = false,
                found => score += found,
            }
        }

        if score == 0 {
            continue;
        }

        let hit = Hit {
            row,
            score,
            matched_everything: found_all,
        };

        if found_all {
            all.push(hit);
        } else {
            any.push(hit);
        }
    }

    let mut results = if all.is_empty() { any } else { all };
    // Nothing reorders by use: a list that moves under the hand is a list
    // nobody can learn.
    results.sort_by(|a, b| b.score.cmp(&a.score).then(a.row.path.cmp(b.row.path)));

    results
}

/// How well one word matches one row.
///
/// Weighted so a label beats a sentence: somebody typing "budget" wants the row
/// called budget before the four that mention one.
fn scores(row: &Row, word: &str) -> u32 {
    let mut score = 0;

    for spelling in spellings(word) {
        if row.path.contains(&spelling) {
            score += 60;
        }
        if row.label.to_lowercase().contains(&spelling) {
            score += 40;
        }
        if row
            .aliases
            .iter()
            .any(|alias| alias.to_lowercase().contains(&spelling))
        {
            score += 30;
        }
        if row.sentence.to_lowercase().contains(&spelling) {
            score += 10;
        }
        if row.page.label().to_lowercase().contains(&spelling) {
            score += 5;
        }
    }

    score
}

/// A word and everything that means the same as it.
fn spellings(word: &str) -> Vec<String> {
    let mut out = vec![word.to_string()];

    for group in SAME {
        if group.contains(&word) {
            out.extend(group.iter().map(|it| it.to_string()));
        }
    }

    out.sort_unstable();
    out.dedup();
    out
}

/// The words of a query that carry meaning.
///
/// "where do rejects go" becomes ["rejects"], which lands on
/// `cull.rejected_folder`. Without this it becomes four words, three of which
/// match half the table.
fn meaningful(query: &str) -> Vec<String> {
    let words: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .filter(|word| !STOP_WORDS.contains(word))
        .map(|word| word.to_string())
        .collect();

    // A query of nothing but stop words is still a query: "how do i" gets the
    // words back rather than nothing at all.
    if words.is_empty() {
        return query
            .to_lowercase()
            .split_whitespace()
            .map(|word| word.to_string())
            .collect();
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The corpus. Every phrase is one somebody would actually type, most of
    /// them taken from a complaint about some viewer or other. It cannot catch
    /// an omission, only a regression — and it is the only mechanism that keeps
    /// a synonym list alive.
    const QUESTIONS: &[(&str, &str)] = &[
        ("blurry thumbnails", "grid_view.thumbnail_resolution"),
        ("thumbnail resolution", "grid_view.thumbnail_resolution"),
        ("where do rejects go", "cull.rejected_folder"),
        ("rejected folder", "cull.rejected_folder"),
        ("why is my raw small", "raw.source"),
        ("develop raw", "raw.source"),
        ("cr3", "raw.pair_with_jpeg"),
        ("raw+jpeg", "raw.pair_with_jpeg"),
        ("color class", "tags.sc_label[0]"),
        ("text too small", "general.text_scaling"),
        ("font size", "general.text_scaling"),
        ("memory", "cache.ram_budget_mb"),
        ("ram budget", "cache.ram_budget_mb"),
        ("resources", "cache.ram_budget_mb"),
        ("vram", "cache.gpu_budget_mb"),
        ("graphics card memory", "cache.gpu_budget_mb"),
        ("cache.ram_budget_mb", "cache.ram_budget_mb"),
        ("shape of the cells", "grid_view.cell_aspect"),
        ("aspect", "grid_view.cell_aspect"),
        ("how many thumbnails across", "grid_view.images_per_row"),
        ("icc profile", "general.output_icc_profile"),
        ("colour management", "general.output_icc_profile"),
        ("slideshow interval", "slideshow.seconds_per_image"),
        ("keyword file", "tags.catalog_file"),
        ("catalogue", "tags.catalog_file"),
        ("decode threads", "cache.decode_threads"),
        ("auto advance", "tags.advance_after_marking"),
        ("white frame", "image_view.frame_size_relative_to_image"),
        ("overlay format", "image_view.overlay_format"),
        ("filmstrip height", "grid_view.filmstrip_height"),
        ("remember last folder", "general.restore_session"),
        ("white balance", "raw.camera_white_balance"),
        ("highlight recovery", "raw.highlight_mode"),
        ("cheat sheet", "fixed.cheat_sheet"),
        ("stutter", "cache.upload_budget_ms"),
    ];

    #[test]
    fn the_index_answers_these_questions() {
        for (query, wanted) in QUESTIONS {
            let hits = find(query);
            assert!(!hits.is_empty(), "\"{query}\" found nothing at all");

            let found = hits.iter().take(4).any(|hit| hit.row.path == *wanted);
            assert!(
                found,
                "\"{query}\" should reach {wanted}; the first four were {:?}",
                hits.iter()
                    .take(4)
                    .map(|hit| hit.row.path)
                    .collect::<Vec<_>>()
            );
        }
    }

    /// A path pasted from a forum post lands on the control and nothing else.
    #[test]
    fn an_exact_path_wins_outright() {
        let hits = find("cache.gpu_budget_mb");

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].row.path, "cache.gpu_budget_mb");
    }

    /// The one answer a search box may never give.
    #[test]
    fn a_search_is_never_empty_when_any_word_lands() {
        let hits = find("blurry zzzznothing");

        assert!(!hits.is_empty());
        assert!(!hits[0].matched_everything);
    }

    #[test]
    fn stop_words_are_dropped() {
        assert_eq!(meaningful("where do the rejects go"), vec!["rejects"]);
        assert_eq!(meaningful("how is my ram"), vec!["ram"]);
    }

    /// A query of nothing but stop words still gets a chance.
    #[test]
    fn a_query_of_nothing_but_stop_words_is_not_thrown_away() {
        assert!(!meaningful("how do i").is_empty());
    }

    #[test]
    fn spellings_fold_the_pairs_nobody_should_guess_between() {
        assert!(spellings("colour").contains(&"color".to_string()));
        assert!(spellings("color").contains(&"colour".to_string()));
        assert!(spellings("ram").contains(&"memory".to_string()));
        assert!(spellings("cr3").contains(&"raw".to_string()));
    }

    /// Nothing reorders by use, so the same query gives the same order.
    #[test]
    fn the_order_is_stable() {
        let first: Vec<&str> = find("memory").iter().map(|hit| hit.row.path).collect();
        let again: Vec<&str> = find("memory").iter().map(|hit| hit.row.path).collect();

        assert_eq!(first, again);
    }
}

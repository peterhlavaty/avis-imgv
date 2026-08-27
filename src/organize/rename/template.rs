//! Turning a template into a file name.
//!
//! Literal text with `{...}` placeholders for the parts that differ, the way a
//! format string works. Everything a file cannot answer expands to nothing, so
//! one template serves a folder where only some of the pictures carry a given
//! tag.

use super::super::Entry;

/// Characters a file name cannot hold on the platforms we run on.
///
/// The union rather than the platform's own list: a folder is copied between
/// machines, and a name that only works on one of them is a name that will
/// break.
const FORBIDDEN: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];

/// Strips what a file name cannot hold.
///
/// Trailing dots and spaces go too: Windows drops them silently, which turns
/// one name into another behind the user's back.
pub(super) fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| !FORBIDDEN.contains(c) && !c.is_control())
        .collect();

    cleaned.trim().trim_end_matches('.').trim().to_string()
}

/// Expands the placeholders in `template` for one file.
///
/// `{{` and `}}` are literal braces, the way they are in a format string. An
/// unknown or unavailable placeholder expands to nothing, so one template can
/// serve a folder where only some of the files carry a given tag.
pub(super) fn render(template: &str, entry: &Entry, counter: usize, digits: usize) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '}' {
            if chars.peek() == Some(&'}') {
                chars.next();
            }
            out.push('}');
            continue;
        }

        if c != '{' {
            out.push(c);
            continue;
        }

        if chars.peek() == Some(&'{') {
            chars.next();
            out.push('{');
            continue;
        }

        let mut token = String::new();
        let mut closed = false;

        for c in chars.by_ref() {
            if c == '}' {
                closed = true;
                break;
            }
            token.push(c);
        }

        // An unclosed brace is a half typed template, not an error worth
        // stopping for; the preview shows what it currently means.
        if !closed {
            out.push('{');
            out.push_str(&token);
            break;
        }

        out.push_str(&expand(&token, entry, counter, digits));
    }

    out
}

/// One placeholder's value.
fn expand(token: &str, entry: &Entry, counter: usize, digits: usize) -> String {
    let token = token.trim();

    if let Some(tag) = token.strip_prefix("tag:") {
        return entry.tag(tag.trim()).unwrap_or_default().to_string();
    }

    match token.to_ascii_lowercase().as_str() {
        "name" => entry.stem().to_string(),
        "ext" => entry.extension(),
        "counter" | "n" => format!("{counter:0digits$}"),
        "date" => entry.captured().map(|at| at.to_date()).unwrap_or_default(),
        "time" => entry.captured().map(|at| at.to_time()).unwrap_or_default(),
        "datetime" => entry
            .captured()
            .map(|at| format!("{}_{}", at.to_date(), at.to_time()))
            .unwrap_or_default(),
        "year" => entry
            .captured()
            .map(|at| format!("{:04}", at.year))
            .unwrap_or_default(),
        "month" => entry
            .captured()
            .map(|at| format!("{:02}", at.month))
            .unwrap_or_default(),
        "day" => entry
            .captured()
            .map(|at| format!("{:02}", at.day))
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// The placeholders, for the help text beside the template box.
pub const PLACEHOLDERS: &[(&str, &str)] = &[
    ("{name}", "the name it has now, without the extension"),
    ("{counter}", "the number, padded to the digits set below"),
    ("{date}", "capture date, as 2024-11-06"),
    ("{time}", "capture time, as 22-07-19"),
    ("{datetime}", "both, joined by an underscore"),
    ("{year} {month} {day}", "parts of the capture date"),
    ("{tag:Name}", "any metadata tag, such as {tag:ISO}"),
    ("{{ }}", "a literal brace"),
];

#[cfg(test)]
mod tests {
    use super::super::super::test_support::entry;
    use super::super::super::CAPTURE_TAG;
    use super::super::{plan, Options, Planned};
    use super::*;

    fn dated(name: &str) -> Entry {
        entry(
            name,
            0,
            &[(CAPTURE_TAG, "2024:11:06 22:07:19"), ("ISO", "400")],
        )
    }

    fn options(template: &str) -> Options {
        Options {
            template: template.to_string(),
            ..Default::default()
        }
    }

    fn names(planned: &[Planned]) -> Vec<String> {
        planned.iter().map(Planned::new_name).collect()
    }

    #[test]
    fn literal_text_comes_through_unchanged() {
        let planned = plan(&[dated("a.jpg")], &options("Holiday"));
        assert_eq!(names(&planned), vec!["Holiday.jpg"]);
    }

    #[test]
    fn the_original_name_can_be_kept_and_added_to() {
        let planned = plan(&[dated("DSCF0001.jpg")], &options("{name}_edited"));
        assert_eq!(names(&planned), vec!["DSCF0001_edited.jpg"]);
    }

    #[test]
    fn the_capture_time_can_be_used_whole_or_in_parts() {
        let entries = [dated("a.jpg")];

        assert_eq!(
            names(&plan(&entries, &options("{date}"))),
            vec!["2024-11-06.jpg"]
        );
        assert_eq!(
            names(&plan(&entries, &options("{time}"))),
            vec!["22-07-19.jpg"]
        );
        assert_eq!(
            names(&plan(&entries, &options("{datetime}"))),
            vec!["2024-11-06_22-07-19.jpg"]
        );
        assert_eq!(
            names(&plan(&entries, &options("{year}-{month}"))),
            vec!["2024-11.jpg"]
        );
    }

    #[test]
    fn any_metadata_tag_can_go_in_the_name() {
        let planned = plan(&[dated("a.jpg")], &options("iso{tag:ISO}"));
        assert_eq!(names(&planned), vec!["iso400.jpg"]);
    }

    #[test]
    fn a_placeholder_the_file_cannot_fill_leaves_a_gap_rather_than_failing() {
        let planned = plan(&[entry("a.jpg", 0, &[])], &options("x{date}{tag:ISO}y"));
        assert_eq!(names(&planned), vec!["xy.jpg"]);
    }

    #[test]
    fn an_unknown_placeholder_expands_to_nothing() {
        let planned = plan(&[dated("a.jpg")], &options("a{nonsense}b"));
        assert_eq!(names(&planned), vec!["ab.jpg"]);
    }

    #[test]
    fn a_doubled_brace_is_a_literal_one() {
        let planned = plan(&[dated("a.jpg")], &options("{{{counter}}}"));
        assert_eq!(names(&planned), vec!["{0001}.jpg"]);
    }

    #[test]
    fn a_half_typed_template_still_previews() {
        let planned = plan(&[dated("a.jpg")], &options("holiday {count"));
        assert_eq!(names(&planned), vec!["holiday {count.jpg"]);
    }

    #[test]
    fn characters_a_file_name_cannot_hold_are_dropped() {
        let planned = plan(&[dated("a.jpg")], &options("a/b:c*d?e\"f<g>h|i"));
        assert_eq!(names(&planned), vec!["abcdefghi.jpg"]);
    }

    #[test]
    fn a_name_that_windows_would_quietly_shorten_is_shortened_here() {
        let planned = plan(&[dated("a.jpg")], &options("holiday. "));
        assert_eq!(names(&planned), vec!["holiday.jpg"]);
    }
}

//! One grammar for every place the viewer builds a line of text about a
//! photograph.
//!
//! There were two. The status bar took `$( • ƒ#Aperture#)` — a fragment that
//! disappears when the tag is missing — and the bulk rename took `{date}_{n}`
//! with a fixed vocabulary of its own. Neither could do what the other did, so
//! the rename could not put a photograph's ISO in a name without going through
//! a differently spelled placeholder, and the status bar could not say a
//! capture date at all.
//!
//! This understands both, which is what makes replacing them safe: every
//! template anybody has already written still means what it meant. `#Tag#` is
//! read as `{tag:Tag}`, and `$( ... )` is an optional group — the literal text
//! inside it goes when what it was decorating is not there, which is the whole
//! reason that syntax exists. A separator you cannot suppress leaves ` • •  • `
//! on every photograph missing two tags.
//!
//! Everything unknown or unavailable expands to nothing rather than to an
//! error. One template has to serve a folder where only some of the pictures
//! carry a lens name, and a viewer that refused to draw a caption over it
//! would be no use at all.

use std::path::Path;

use crate::metadata::datetime::Timestamp;
use crate::metadata::xmp::Xmp;
use crate::metadata::Metadata;

/// What a template is being rendered about.
///
/// Every field is optional because the callers know different things: the
/// rename has a counter and no marks in front of it, the status bar has marks
/// and no counter, and a caption under a thumbnail may have neither yet.
#[derive(Default, Clone, Copy)]
pub struct Subject<'a> {
    pub path: Option<&'a Path>,
    pub metadata: Option<&'a Metadata>,
    pub annotations: Option<&'a Xmp>,
    /// The number and how many digits to pad it to, for a rename.
    pub counter: Option<(usize, usize)>,
    /// Size on disk, when the caller happens to know it.
    pub size: Option<u64>,
}

impl<'a> Subject<'a> {
    pub fn new(path: &'a Path) -> Subject<'a> {
        Subject {
            path: Some(path),
            ..Subject::default()
        }
    }

    pub fn with_metadata(mut self, metadata: &'a Metadata) -> Subject<'a> {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_annotations(mut self, annotations: &'a Xmp) -> Subject<'a> {
        self.annotations = Some(annotations);
        self
    }

    pub fn with_counter(mut self, counter: usize, digits: usize) -> Subject<'a> {
        self.counter = Some((counter, digits));
        self
    }

    pub fn with_size(mut self, size: u64) -> Subject<'a> {
        self.size = Some(size);
        self
    }

    fn tag(&self, name: &str) -> Option<&str> {
        self.metadata?.tags.get(name).map(String::as_str)
    }

    fn captured(&self) -> Option<Timestamp> {
        Timestamp::parse(self.tag(crate::organize::CAPTURE_TAG)?)
    }
}

/// Expands `template` for `subject`.
pub fn render(template: &str, subject: &Subject<'_>) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // `$( ... )` — kept only if everything it interpolates resolves.
            '$' if chars.peek() == Some(&'(') => {
                chars.next();
                let body = take_group(&mut chars);
                out.push_str(&optional(&body, subject));
            }
            '}' => {
                // `}}` is a literal brace, the way a format string has it.
                if chars.peek() == Some(&'}') {
                    chars.next();
                }
                out.push('}');
            }
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                out.push('{');
            }
            '{' => {
                let (token, closed) = take_token(&mut chars);

                // An unclosed brace is a half typed template, not an error
                // worth stopping for: the preview shows what it means so far.
                if !closed {
                    out.push('{');
                    out.push_str(&token);
                    break;
                }

                out.push_str(&expand(&token, subject).unwrap_or_default());
            }
            _ => out.push(c),
        }
    }

    out
}

/// The body of a `$( ... )` group, up to its closing bracket.
///
/// Nesting is not supported and is not wanted: a group exists to drop one
/// separator and one value, and a grammar people write by hand in a
/// configuration file should be one they can hold in their head.
fn take_group(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut body = String::new();

    for c in chars.by_ref() {
        if c == ')' {
            break;
        }
        body.push(c);
    }

    body
}

/// A placeholder name, and whether its brace was closed.
fn take_token(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> (String, bool) {
    let mut token = String::new();

    for c in chars.by_ref() {
        if c == '}' {
            return (token, true);
        }
        token.push(c);
    }

    (token, false)
}

/// A `$( ... )` group: its text, or nothing at all.
///
/// Nothing at all when any placeholder inside it is empty, which is the point:
/// the literal text in a group is there to decorate a value, and decoration
/// without the value is noise. A group with no placeholder in it is literal
/// text and always kept.
fn optional(body: &str, subject: &Subject<'_>) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // The older spelling: `#Tag#` is `{tag:Tag}`.
            '#' => {
                let mut name = String::new();
                let mut closed = false;

                for c in chars.by_ref() {
                    if c == '#' {
                        closed = true;
                        break;
                    }
                    name.push(c);
                }

                if !closed {
                    out.push('#');
                    out.push_str(&name);
                    continue;
                }

                match subject.tag(name.trim()).filter(|value| !value.is_empty()) {
                    Some(value) => out.push_str(value),
                    None => return String::new(),
                }
            }
            '{' => {
                let (token, closed) = take_token(&mut chars);
                if !closed {
                    out.push('{');
                    out.push_str(&token);
                    continue;
                }

                match expand(&token, subject).filter(|value| !value.is_empty()) {
                    Some(value) => out.push_str(&value),
                    None => return String::new(),
                }
            }
            _ => out.push(c),
        }
    }

    out
}

/// One placeholder's value, or `None` when the photograph cannot answer it.
fn expand(token: &str, subject: &Subject<'_>) -> Option<String> {
    let token = token.trim();

    if let Some(name) = token.strip_prefix("tag:") {
        return subject.tag(name.trim()).map(str::to_string);
    }

    let path = subject.path;
    let at = subject.captured();

    let value = match token.to_ascii_lowercase().as_str() {
        // The file itself.
        "name" => stem(path?),
        "ext" => extension(path?),
        "folder" => path?
            .parent()
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned())?,
        "size" => human_size(subject.size?),

        // The rename's counter.
        "counter" | "n" => {
            let (counter, digits) = subject.counter?;
            format!("{counter:0digits$}")
        }

        // When the shutter opened.
        "date" => at?.to_date(),
        "time" => at?.to_time(),
        "datetime" => {
            let at = at?;
            format!("{}_{}", at.to_date(), at.to_time())
        }
        "year" => format!("{:04}", at?.year),
        "month" => format!("{:02}", at?.month),
        "day" => format!("{:02}", at?.day),
        "hour" => format!("{:02}", at?.hour),
        "minute" => format!("{:02}", at?.minute),
        "second" => format!("{:02}", at?.second),

        // The tags people actually ask for, spelled the way they say them
        // rather than the way EXIF does.
        "iso" => subject.tag("ISO")?.to_string(),
        "aperture" => subject.tag("Aperture")?.to_string(),
        "shutter" => subject.tag("Shutter Speed")?.to_string(),
        "focal" => subject.tag("Focal Length")?.to_string(),
        "lens" => subject
            .tag("Lens Model")
            .or(subject.tag("Lens"))?
            .to_string(),
        "camera" => subject.tag("Camera Model Name")?.to_string(),
        "dimensions" => subject.tag("Image Size")?.to_string(),

        // What the user put on it.
        "stars" => {
            let stars = subject.annotations?.stars();
            (stars > 0).then(|| "★".repeat(stars as usize))?
        }
        "rating" => {
            let stars = subject.annotations?.stars();
            (stars > 0).then(|| stars.to_string())?
        }
        "flag" => {
            let flag = subject.annotations?.flag();
            (flag != crate::metadata::xmp::Flag::Unflagged).then(|| flag.glyph().to_string())?
        }
        "label" => subject.annotations?.label.clone()?,
        "keywords" => {
            let keywords = &subject.annotations?.keywords;
            (!keywords.is_empty()).then(|| keywords.join(", "))?
        }

        _ => return None,
    };

    Some(value)
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn extension(path: &Path) -> String {
    path.extension()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// A size somebody can read, which is what a caption wants.
fn human_size(bytes: u64) -> String {
    const UNITS: &[(&str, u64)] = &[("GB", 1 << 30), ("MB", 1 << 20), ("kB", 1 << 10)];

    for (unit, size) in UNITS {
        if bytes >= *size {
            return format!("{:.1} {unit}", bytes as f64 / *size as f64);
        }
    }

    format!("{bytes} B")
}

/// The vocabulary, for the help beside a template box.
pub const PLACEHOLDERS: &[(&str, &str)] = &[
    ("{name}", "the name it has now, without the extension"),
    ("{ext}", "the extension"),
    ("{folder}", "the folder it is in"),
    ("{counter}", "the number, padded to the digits set below"),
    ("{date}", "capture date, as 2024-11-06"),
    ("{time}", "capture time, as 22-07-19"),
    ("{datetime}", "both, joined by an underscore"),
    ("{year} {month} {day}", "parts of the capture date"),
    ("{hour} {minute} {second}", "parts of the capture time"),
    ("{iso} {aperture} {shutter}", "how it was exposed"),
    ("{focal} {lens} {camera}", "what it was taken with"),
    ("{dimensions} {size}", "how big it is"),
    ("{stars} {rating} {flag} {label}", "what you put on it"),
    ("{keywords}", "its keywords, comma separated"),
    ("{tag:Name}", "any metadata tag, such as {tag:ISO}"),
    ("$( • {iso})", "kept only when what is inside it resolves"),
    ("{{ }}", "a literal brace"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn metadata(tags: &[(&str, &str)]) -> Metadata {
        let mut found = Metadata::default();
        for (name, value) in tags {
            found.tags.insert(name.to_string(), value.to_string());
        }

        found
    }

    fn path() -> PathBuf {
        PathBuf::from("/photos/trip/IMG_1234.JPG")
    }

    #[test]
    fn literal_text_is_left_alone() {
        let at = path();
        assert_eq!(render("just words", &Subject::new(&at)), "just words");
    }

    #[test]
    fn the_file_answers_for_itself() {
        let at = path();
        let subject = Subject::new(&at);

        assert_eq!(render("{name}", &subject), "IMG_1234");
        assert_eq!(render("{ext}", &subject), "JPG");
        assert_eq!(render("{folder}", &subject), "trip");
    }

    #[test]
    fn the_capture_time_comes_apart() {
        let at = path();
        let found = metadata(&[(crate::organize::CAPTURE_TAG, "2024:11:06 22:07:19")]);
        let subject = Subject::new(&at).with_metadata(&found);

        assert_eq!(render("{date}", &subject), "2024-11-06");
        assert_eq!(render("{time}", &subject), "22-07-19");
        assert_eq!(render("{datetime}", &subject), "2024-11-06_22-07-19");
        assert_eq!(render("{year}/{month}/{day}", &subject), "2024/11/06");
        assert_eq!(render("{hour}:{minute}:{second}", &subject), "22:07:19");
    }

    #[test]
    fn a_counter_is_padded_to_what_was_asked_for() {
        let at = path();
        let subject = Subject::new(&at).with_counter(7, 4);

        assert_eq!(render("{counter}", &subject), "0007");
        assert_eq!(render("{n}", &subject), "0007");
    }

    /// The tags people ask for by their own names rather than EXIF's.
    #[test]
    fn the_common_tags_have_names_people_use() {
        let at = path();
        let found = metadata(&[
            ("ISO", "400"),
            ("Aperture", "2.8"),
            ("Shutter Speed", "1/250"),
            ("Lens Model", "50mm"),
            ("Camera Model Name", "R5"),
        ]);
        let subject = Subject::new(&at).with_metadata(&found);

        assert_eq!(render("{iso}", &subject), "400");
        assert_eq!(render("{aperture}", &subject), "2.8");
        assert_eq!(render("{shutter}", &subject), "1/250");
        assert_eq!(render("{lens}", &subject), "50mm");
        assert_eq!(render("{camera}", &subject), "R5");
    }

    #[test]
    fn any_tag_at_all_is_reachable() {
        let at = path();
        let found = metadata(&[("Some Odd Tag", "value")]);
        let subject = Subject::new(&at).with_metadata(&found);

        assert_eq!(render("{tag:Some Odd Tag}", &subject), "value");
    }

    #[test]
    fn the_marks_are_reachable_too() {
        let at = path();
        let marks = Xmp {
            rating: 4,
            picked: true,
            label: Some("Red".to_string()),
            keywords: vec!["Tatras".to_string(), "Winter".to_string()],
            hierarchy: Vec::new(),
            ..Default::default()
        };
        let subject = Subject::new(&at).with_annotations(&marks);

        assert_eq!(render("{rating}", &subject), "4");
        assert_eq!(render("{stars}", &subject), "★★★★");
        assert_eq!(render("{label}", &subject), "Red");
        assert_eq!(render("{keywords}", &subject), "Tatras, Winter");
        assert!(!render("{flag}", &subject).is_empty());
    }

    /// Everything a photograph cannot answer disappears, so one template
    /// serves a folder where only some of them carry a lens name.
    #[test]
    fn what_cannot_be_answered_disappears() {
        let at = path();
        let subject = Subject::new(&at);

        assert_eq!(render("[{iso}]", &subject), "[]");
        assert_eq!(render("{nonsense}", &subject), "");
        assert_eq!(render("{tag:Missing}", &subject), "");
    }

    /// The reason optional groups exist: a separator you cannot suppress
    /// leaves a line of them on a photograph that answers nothing.
    #[test]
    fn an_optional_group_goes_with_what_it_decorated() {
        let at = path();
        let found = metadata(&[("ISO", "400")]);
        let subject = Subject::new(&at).with_metadata(&found);

        assert_eq!(
            render("{name}$( • {iso} ISO)", &subject),
            "IMG_1234 • 400 ISO"
        );
        assert_eq!(render("{name}$( • {lens})", &subject), "IMG_1234");
    }

    /// Every template anybody has already written still means what it meant.
    #[test]
    fn the_older_spelling_still_works() {
        let at = path();
        let found = metadata(&[("Aperture", "1.8"), ("ISO", "100")]);
        let subject = Subject::new(&at).with_metadata(&found);

        let shipped = "$(#File Name#)$( • ƒ#Aperture#)$( • #Shutter Speed#)$( • #ISO# ISO)";
        let with_name = metadata(&[
            ("File Name", "IMG_1234.JPG"),
            ("Aperture", "1.8"),
            ("ISO", "100"),
        ]);
        let subject_with_name = Subject::new(&at).with_metadata(&with_name);

        // The shutter speed is missing, and its separator goes with it.
        assert_eq!(
            render(shipped, &subject_with_name),
            "IMG_1234.JPG • ƒ1.8 • 100 ISO"
        );

        // And the two spellings agree.
        assert_eq!(
            render("$( • #ISO#)", &subject),
            render("$( • {tag:ISO})", &subject)
        );
    }

    #[test]
    fn a_group_of_literal_text_is_kept() {
        let at = path();
        assert_eq!(render("$(always)", &Subject::new(&at)), "always");
    }

    #[test]
    fn braces_can_be_written_literally() {
        let at = path();
        assert_eq!(render("{{name}}", &Subject::new(&at)), "{name}");
    }

    /// A half typed template is shown as far as it goes rather than refused.
    #[test]
    fn an_unclosed_placeholder_is_not_an_error() {
        let at = path();
        assert_eq!(render("{name}_{dat", &Subject::new(&at)), "IMG_1234_{dat");
    }

    #[test]
    fn a_size_is_written_for_people() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(2 * 1024), "2.0 kB");
        assert_eq!(human_size(3 * 1024 * 1024), "3.0 MB");
        assert_eq!(human_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn every_placeholder_in_the_help_resolves_or_is_syntax() {
        let at = path();
        let found = metadata(&[
            (crate::organize::CAPTURE_TAG, "2024:11:06 22:07:19"),
            ("ISO", "400"),
            ("Aperture", "2.8"),
            ("Shutter Speed", "1/250"),
            ("Focal Length", "50 mm"),
            ("Lens Model", "50mm"),
            ("Camera Model Name", "R5"),
            ("Image Size", "6000x4000"),
        ]);
        let marks = Xmp {
            rating: 3,
            picked: true,
            label: Some("Red".to_string()),
            keywords: vec!["One".to_string()],
            hierarchy: Vec::new(),
            ..Default::default()
        };

        let subject = Subject::new(&at)
            .with_metadata(&found)
            .with_annotations(&marks)
            .with_counter(1, 3)
            .with_size(1234);

        for (spelling, meaning) in PLACEHOLDERS {
            assert!(!meaning.is_empty(), "{spelling} has no description");

            // The syntax entries are not placeholders to resolve.
            if spelling.contains("{{") || spelling.starts_with("$(") || spelling.contains("Name") {
                continue;
            }

            for token in spelling.split_whitespace() {
                assert!(
                    !render(token, &subject).is_empty(),
                    "{token} resolved to nothing"
                );
            }
        }
    }
}

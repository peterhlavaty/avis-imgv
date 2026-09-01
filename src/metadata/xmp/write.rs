//! Writing a rating and keywords back into an XMP document.
//!
//! An existing document is edited rather than replaced: sidecars are routinely
//! shared with a raw converter that keeps its entire develop history in there,
//! and overwriting it would be destroying work.

use std::io::Cursor;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{NsReader, Writer};

use super::{
    namespace_of, Namespace, Orientation, Xmp, NS_DC, NS_DIGIKAM, NS_LIGHTROOM, NS_RDF, NS_TIFF,
    NS_XMP,
};

/// Identifies documents this viewer wrote, in the toolkit field every XMP
/// writer stamps.
pub const MARKER: &str = "avis-imgv";

/// A document with more events than this is not one we should be rewriting.
const MAX_EVENTS: usize = 100_000;

type Sink = Writer<Cursor<Vec<u8>>>;

/// Produces a document carrying `xmp`, editing `existing` when there is one.
///
/// Everything the viewer does not understand is passed through untouched and
/// the original formatting is preserved, so a sidecar shared with another tool
/// comes back recognisable.
///
/// Returns nothing when there is a document that cannot be rewritten without
/// losing what it holds. A sidecar routinely carries an entire develop history,
/// so refusing to write is the only safe answer to "I could not read this":
/// the caller reports it and the file stays as it was.
pub fn update(existing: Option<&str>, xmp: &Xmp) -> Option<String> {
    match existing {
        // Nothing there, or a file somebody created and left blank.
        None => Some(fresh(xmp)),
        Some(document) if document.trim().is_empty() => Some(fresh(xmp)),
        Some(document) => edit(document, xmp),
    }
}

/// Rewrites an existing document, or gives up rather than write over it.
///
/// Gives up when the document has no `rdf:Description` to edit, when it does
/// not parse, and when it is longer than [`MAX_EVENTS`] — the last of those
/// used to fall through and hand back the half of it that had been written.
fn edit(document: &str, xmp: &Xmp) -> Option<String> {
    let mut reader = NsReader::from_str(document);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Our two properties are stripped wherever they appear and reinstated in
    // the first description, so a document cannot end up saying two things.
    let mut skipping = 0usize;
    let mut written = false;
    let mut finished = false;

    for _ in 0..MAX_EVENTS {
        let (namespace, event) = {
            let (resolved, event) = reader.read_resolved_event().ok()?;
            (namespace_of(&resolved), event)
        };

        if skipping > 0 {
            match event {
                Event::Start(_) => skipping += 1,
                Event::End(_) => skipping -= 1,
                Event::Eof => return None,
                _ => {}
            }
            continue;
        }

        match event {
            Event::Eof => {
                finished = true;
                break;
            }
            Event::Start(ref element) | Event::Empty(ref element) => {
                let local = element.local_name();
                let name: &str = local.as_ref();

                if is_ours_named(namespace, name) {
                    // An empty element has no subtree to skip.
                    if matches!(event, Event::Start(_)) {
                        skipping = 1;
                    }
                    continue;
                }

                if namespace == Namespace::Rdf && name == "Description" {
                    let element = element.to_owned();
                    let self_closing = matches!(event, Event::Empty(_));

                    if written {
                        // A later description keeps everything it had except
                        // our properties, which the first one now carries.
                        strip_ours(&mut writer, &mut reader, &element, self_closing).ok()?;
                    } else {
                        written = true;
                        write_description(&mut writer, &mut reader, &element, xmp, self_closing)
                            .ok()?;
                    }
                    continue;
                }

                writer.write_event(event).ok()?;
            }
            other => writer.write_event(other).ok()?,
        }
    }

    if !finished || !written {
        return None;
    }

    String::from_utf8(writer.into_inner().into_inner()).ok()
}

/// Writes a description through, minus the properties the viewer owns.
///
/// A sidecar with more than one `rdf:Description` used to come back with a
/// rating on each of them, and the reader would find the stale one first.
fn strip_ours(
    writer: &mut Sink,
    reader: &mut NsReader<&[u8]>,
    element: &BytesStart<'_>,
    self_closing: bool,
) -> std::io::Result<()> {
    let name: String = element.name().as_ref().to_string();
    let mut rebuilt = BytesStart::new(name);

    for attribute in element.attributes().flatten() {
        if is_ours(reader, &attribute) {
            continue;
        }

        rebuilt.push_attribute(attribute);
    }

    let event = if self_closing {
        Event::Empty(rebuilt)
    } else {
        Event::Start(rebuilt)
    };

    writer.write_event(event)
}

/// Whether an attribute is one of the properties the viewer maintains.
fn is_ours(
    reader: &mut NsReader<&[u8]>,
    attribute: &quick_xml::events::attributes::Attribute,
) -> bool {
    let (resolved, local) = reader.resolver_mut().resolve_attribute(attribute.key);

    is_ours_named(namespace_of(&resolved), local.as_ref())
}

/// The same question, asked of an element rather than an attribute.
fn is_ours_named(namespace: Namespace, name: &str) -> bool {
    matches!(
        (namespace, name),
        (Namespace::Xmp, "Rating")
            | (Namespace::Xmp, "Label")
            | (Namespace::Tiff, "Orientation")
            | (Namespace::Dc, "subject")
            | (Namespace::Lightroom, "hierarchicalSubject")
            | (Namespace::DigiKam, "PickLabel")
    )
}

/// Writes the description element with our rating on it, followed by the
/// keyword bag.
fn write_description(
    writer: &mut Sink,
    reader: &mut NsReader<&[u8]>,
    element: &BytesStart<'_>,
    xmp: &Xmp,
    self_closing: bool,
) -> std::io::Result<()> {
    let name: String = element.name().as_ref().to_string();
    let mut rebuilt = BytesStart::new(name.clone());
    let mut declares_xmp = false;
    let mut declares_digikam = false;
    let mut declares_tiff = false;

    for attribute in element.attributes().flatten() {
        // Drop anything already there that we are about to write, whatever
        // prefix it used.
        if is_ours(reader, &attribute) {
            continue;
        }

        declares_xmp |= attribute.key.as_ref() == "xmlns:xmp";
        declares_digikam |= attribute.key.as_ref() == "xmlns:digiKam";
        declares_tiff |= attribute.key.as_ref() == "xmlns:tiff";
        rebuilt.push_attribute(attribute);
    }

    // The prefixes have to be in scope for the attributes we are about to add;
    // declaring one again on this element is harmless if it already is.
    if !declares_xmp {
        rebuilt.push_attribute(("xmlns:xmp", NS_XMP));
    }
    if xmp.picked && !declares_digikam {
        rebuilt.push_attribute(("xmlns:digiKam", NS_DIGIKAM));
    }

    let turned = xmp.orientation != Orientation::Normal;
    if turned && !declares_tiff {
        rebuilt.push_attribute(("xmlns:tiff", NS_TIFF));
    }

    let rating = xmp.rating.to_string();
    rebuilt.push_attribute(("xmp:Rating", rating.as_str()));

    if let Some(label) = &xmp.label {
        rebuilt.push_attribute(("xmp:Label", escape(label).as_str()));
    }

    // Only written when it is set: an unpicked frame should not leave a
    // digiKam property behind in a sidecar that never had one.
    if xmp.picked {
        rebuilt.push_attribute(("digiKam:PickLabel", "3"));
    }

    // Likewise: a photograph nobody has turned carries no orientation, so a
    // sidecar written by this viewer says nothing about how a file it never
    // turned should be drawn.
    let orientation = xmp.orientation.to_exif().to_string();
    if turned {
        rebuilt.push_attribute(("tiff:Orientation", orientation.as_str()));
    }

    writer.write_event(Event::Start(rebuilt))?;
    write_keywords(writer, &xmp.keywords)?;
    write_hierarchy(writer, &xmp.hierarchy)?;

    // A self-closing description has to become a pair now that it has content.
    if self_closing {
        writer.write_event(Event::End(BytesEnd::new(name)))?;
    }

    Ok(())
}

/// Writes `dc:subject` as an unordered bag, the form every reader understands.
fn write_keywords(writer: &mut Sink, keywords: &[String]) -> std::io::Result<()> {
    write_bag(writer, "dc:subject", "xmlns:dc", NS_DC, keywords)
}

/// And `lr:hierarchicalSubject`, for the readers that understand paths.
///
/// Beside the flat list rather than instead of it: a program that knows about
/// hierarchies reads this one, and a program that does not still finds the
/// keyword in `dc:subject`. Writing only the paths would leave the second kind
/// seeing nothing at all.
fn write_hierarchy(writer: &mut Sink, paths: &[String]) -> std::io::Result<()> {
    write_bag(
        writer,
        "lr:hierarchicalSubject",
        "xmlns:lr",
        NS_LIGHTROOM,
        paths,
    )
}

/// One bag of strings under a named element.
fn write_bag(
    writer: &mut Sink,
    element: &str,
    prefix: &str,
    namespace: &str,
    items: &[String],
) -> std::io::Result<()> {
    if items.is_empty() {
        return Ok(());
    }

    // Both elements carry their own namespace declaration so the block stays
    // valid wherever it is inserted.
    let mut opening = BytesStart::new(element);
    opening.push_attribute((prefix, namespace));
    writer.write_event(Event::Start(opening))?;

    let mut bag = BytesStart::new("rdf:Bag");
    bag.push_attribute(("xmlns:rdf", NS_RDF));
    writer.write_event(Event::Start(bag))?;

    for item in items {
        writer.write_event(Event::Start(BytesStart::new("rdf:li")))?;
        writer.write_event(Event::Text(BytesText::new(item)))?;
        writer.write_event(Event::End(BytesEnd::new("rdf:li")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("rdf:Bag")))?;
    writer.write_event(Event::End(BytesEnd::new(element)))
}

/// A complete document, for a file that has no sidecar yet.
fn fresh(xmp: &Xmp) -> String {
    let bag = |element: &str, values: &[String]| {
        if values.is_empty() {
            return String::new();
        }

        let items: String = values
            .iter()
            .map(|value| format!("     <rdf:li>{}</rdf:li>\n", escape(value)))
            .collect();

        format!("   <{element}>\n    <rdf:Bag>\n{items}    </rdf:Bag>\n   </{element}>\n")
    };

    let subject = format!(
        "{}{}",
        bag("dc:subject", &xmp.keywords),
        bag("lr:hierarchicalSubject", &xmp.hierarchy)
    );

    let lightroom_ns = if xmp.hierarchy.is_empty() {
        String::new()
    } else {
        format!("\n    xmlns:lr=\"{NS_LIGHTROOM}\"")
    };

    let label = match &xmp.label {
        Some(label) => format!("\n   xmp:Label=\"{}\"", escape(label)),
        None => String::new(),
    };

    let (tiff_ns, turn) = if xmp.orientation == Orientation::Normal {
        (String::new(), String::new())
    } else {
        (
            format!("\n    xmlns:tiff=\"{NS_TIFF}\""),
            format!("\n   tiff:Orientation=\"{}\"", xmp.orientation.to_exif()),
        )
    };

    let (digikam_ns, pick) = if xmp.picked {
        (
            format!("\n    xmlns:digiKam=\"{NS_DIGIKAM}\""),
            "\n   digiKam:PickLabel=\"3\"".to_string(),
        )
    } else {
        (String::new(), String::new())
    };

    format!(
        r#"<?xpacket begin="{bom}" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="{MARKER}">
 <rdf:RDF xmlns:rdf="{NS_RDF}">
  <rdf:Description rdf:about=""
    xmlns:xmp="{NS_XMP}"
    xmlns:dc="{NS_DC}"{digikam_ns}{lightroom_ns}{tiff_ns}
   xmp:Rating="{rating}"{label}{pick}{turn}>
{subject}  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>
"#,
        bom = '\u{feff}',
        rating = xmp.rating,
    )
}

fn escape(text: &str) -> String {
    quick_xml::escape::escape(text).into_owned()
}

#[cfg(test)]
mod tests {
    use super::super::read;
    use super::*;

    fn xmp(rating: i8, keywords: &[&str]) -> Xmp {
        Xmp {
            rating,
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
            ..Xmp::default()
        }
    }

    /// The document `update` produces, for the cases that must produce one.
    fn updated(existing: Option<&str>, xmp: &Xmp) -> String {
        update(existing, xmp).expect("a document")
    }

    /// Both forms are written, which is the whole point: a program that
    /// understands paths reads one and a program that does not reads the
    /// other.
    #[test]
    fn a_hierarchy_is_written_beside_the_flat_keywords() {
        let xmp = Xmp {
            rating: 3,
            keywords: vec!["Tatras".to_string()],
            hierarchy: vec!["Places|Slovakia|Tatras".to_string()],
            ..Xmp::default()
        };

        let written = updated(None, &xmp);

        assert!(written.contains("<dc:subject>"), "{written}");
        assert!(written.contains("lr:hierarchicalSubject"), "{written}");
        assert!(written.contains("Places|Slovakia|Tatras"), "{written}");
        assert!(written.contains(NS_LIGHTROOM), "the namespace is declared");

        let read = read::read(&written).expect("it reads back");
        assert_eq!(read.keywords, xmp.keywords);
        assert_eq!(read.hierarchy, xmp.hierarchy);
    }

    /// And editing a document that already has one replaces it rather than
    /// leaving two.
    #[test]
    fn an_existing_hierarchy_is_replaced_not_duplicated() {
        let first = updated(
            None,
            &Xmp {
                hierarchy: vec!["Places|Slovakia".to_string()],
                ..Xmp::default()
            },
        );

        let second = updated(
            Some(&first),
            &Xmp {
                hierarchy: vec!["Places|Austria".to_string()],
                ..Xmp::default()
            },
        );

        assert_eq!(
            second.matches("lr:hierarchicalSubject").count(),
            2,
            "one opening and one closing tag: {second}"
        );
        assert!(!second.contains("Slovakia"), "{second}");
        assert!(second.contains("Austria"), "{second}");
    }

    /// A photograph with no hierarchy writes none, rather than an empty bag
    /// and a namespace nothing uses.
    #[test]
    fn no_hierarchy_writes_no_element() {
        let written = updated(
            None,
            &Xmp {
                keywords: vec!["Autumn".to_string()],
                ..Xmp::default()
            },
        );

        assert!(!written.contains("hierarchicalSubject"), "{written}");
        assert!(!written.contains(NS_LIGHTROOM), "{written}");
    }

    #[test]
    fn a_fresh_document_round_trips() {
        let written = updated(None, &xmp(4, &["Slovakia", "Autumn"]));
        let back = read(&written).expect("reads back");

        assert_eq!(back.rating, 4);
        assert_eq!(back.keywords, vec!["Slovakia", "Autumn"]);
        assert!(written.contains(MARKER));
    }

    #[test]
    fn an_empty_annotation_still_produces_a_document() {
        let written = updated(None, &Xmp::default());

        assert!(read(&written).expect("reads back").is_empty());
    }

    #[test]
    fn keywords_are_escaped() {
        let written = updated(None, &xmp(0, &["black & white", "<odd>"]));

        assert_eq!(
            read(&written).unwrap().keywords,
            vec!["black & white", "<odd>"]
        );
    }

    #[test]
    fn editing_replaces_what_was_there() {
        let first = updated(None, &xmp(1, &["Draft"]));
        let second = updated(Some(&first), &xmp(5, &["Final", "Print"]));
        let back = read(&second).unwrap();

        assert_eq!(back.rating, 5);
        assert_eq!(back.keywords, vec!["Final", "Print"]);
        assert_eq!(
            second.matches("dc:subject").count(),
            2,
            "one opening tag and one closing tag, not two of each"
        );
    }

    #[test]
    fn editing_preserves_what_another_tool_wrote() {
        let existing = format!(
            r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Some Raw Converter">
 <rdf:RDF xmlns:rdf="{NS_RDF}">
  <rdf:Description rdf:about="" xmlns:xmp="{NS_XMP}" xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    xmp:Rating="1" crs:Exposure2012="+0.35">
   <crs:ToneCurve><rdf:Seq><rdf:li>0, 0</rdf:li></rdf:Seq></crs:ToneCurve>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#
        );

        let edited = updated(Some(&existing), &xmp(3, &["Keeper"]));
        let back = read(&edited).unwrap();

        assert!(edited.contains(r#"crs:Exposure2012="+0.35""#));
        assert!(edited.contains("crs:ToneCurve"));
        assert!(edited.contains("Some Raw Converter"));
        assert_eq!(back.rating, 3);
        assert_eq!(back.keywords, vec!["Keeper"]);
    }

    #[test]
    fn a_self_closing_description_gains_a_body() {
        let existing =
            format!(r#"<rdf:RDF xmlns:rdf="{NS_RDF}"><rdf:Description rdf:about=""/></rdf:RDF>"#);

        let back = read(&updated(Some(&existing), &xmp(2, &["Tagged"]))).unwrap();

        assert_eq!(back.rating, 2);
        assert_eq!(back.keywords, vec!["Tagged"]);
    }

    #[test]
    fn a_rating_stored_as_an_element_is_replaced_too() {
        let existing = format!(
            r#"<rdf:RDF xmlns:rdf="{NS_RDF}"><rdf:Description xmlns:xmp="{NS_XMP}">
                 <xmp:Rating>1</xmp:Rating>
               </rdf:Description></rdf:RDF>"#
        );

        let edited = updated(Some(&existing), &xmp(4, &[]));

        assert_eq!(read(&edited).unwrap().rating, 4);
        assert!(!edited.contains("<xmp:Rating>"));
    }

    /// A document that cannot be edited is one whose contents we would be
    /// throwing away, so nothing is produced and the caller leaves it alone.
    #[test]
    fn something_that_is_not_xmp_is_left_alone() {
        assert!(update(Some("this is not a document"), &xmp(3, &[])).is_none());
        assert!(update(Some("<other>not xmp at all</other>"), &xmp(3, &[])).is_none());
    }

    #[test]
    fn a_blank_document_is_written_afresh() {
        let written = update(Some("  \n\t "), &xmp(3, &[])).expect("a document");

        assert_eq!(read(&written).unwrap().rating, 3);
        assert!(written.contains(MARKER));
    }

    /// The event budget used to be a truncation: the half of the document that
    /// had been written was handed back and saved over the whole of it.
    #[test]
    fn a_document_longer_than_the_budget_is_refused_rather_than_truncated() {
        let filler: String = (0..MAX_EVENTS)
            .map(|i| format!("<crs:li>{i}</crs:li>"))
            .collect();

        let existing = format!(
            r#"<rdf:RDF xmlns:rdf="{NS_RDF}"><rdf:Description xmlns:xmp="{NS_XMP}"
                 xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
                 xmp:Rating="1">{filler}</rdf:Description></rdf:RDF>"#
        );

        assert!(update(Some(&existing), &xmp(4, &[])).is_none());
    }

    /// The rating attribute used to be stripped from the first description
    /// only, leaving a second one to contradict it.
    #[test]
    fn a_second_description_does_not_keep_a_stale_rating() {
        let existing = format!(
            r#"<rdf:RDF xmlns:rdf="{NS_RDF}">
                 <rdf:Description rdf:about="" xmlns:xmp="{NS_XMP}" xmp:Rating="1"/>
                 <rdf:Description rdf:about="" xmlns:xmp="{NS_XMP}" xmp:Rating="2"/>
               </rdf:RDF>"#
        );

        let edited = updated(Some(&existing), &xmp(5, &[]));

        assert_eq!(edited.matches("xmp:Rating=").count(), 1, "{edited}");
        assert_eq!(read(&edited).unwrap().rating, 5);
    }

    #[test]
    fn clearing_a_rating_writes_a_zero_rather_than_forgetting() {
        let rated = updated(None, &xmp(5, &["Keep"]));
        let cleared = updated(Some(&rated), &Xmp::default());
        let back = read(&cleared).unwrap();

        assert_eq!(back.rating, 0);
        assert!(back.keywords.is_empty());
    }

    /// A turn goes to the sidecar and the photograph is not touched.
    #[test]
    fn a_turn_round_trips() {
        let turned = Xmp {
            orientation: Orientation::Rotate90Cw,
            ..Default::default()
        };

        let written = updated(None, &turned);
        assert!(written.contains("tiff:Orientation=\"6\""), "{written}");
        assert_eq!(read(&written).unwrap(), turned);
    }

    /// And a photograph nobody has turned says nothing about orientation, so a
    /// sidecar this viewer writes does not start claiming to know how a file
    /// it never turned should be drawn.
    #[test]
    fn an_unturned_photograph_writes_no_orientation() {
        let plain = Xmp {
            rating: 3,
            ..Default::default()
        };

        let written = updated(None, &plain);
        assert!(!written.contains("tiff:Orientation"), "{written}");
    }

    /// A turn taken back leaves nothing behind either.
    #[test]
    fn turning_back_to_upright_takes_the_property_out() {
        let turned = Xmp {
            orientation: Orientation::Rotate90Cw,
            ..Default::default()
        };

        let written = updated(None, &turned);
        let back = Xmp::default();
        let again = updated(Some(&written), &back);

        assert!(!again.contains("tiff:Orientation"), "{again}");
        assert_eq!(read(&again).unwrap(), back);
    }

    #[test]
    fn a_label_and_a_pick_round_trip() {
        let marked = Xmp {
            rating: 2,
            picked: true,
            label: Some("Green".to_string()),
            keywords: vec!["Keeper".to_string()],
            hierarchy: Vec::new(),
            ..Default::default()
        };

        let written = updated(None, &marked);
        assert_eq!(read(&written).unwrap(), marked);

        // And again through an existing document rather than a fresh one.
        let edited = updated(Some(&written), &marked);
        assert_eq!(read(&edited).unwrap(), marked);
    }

    #[test]
    fn a_rejection_round_trips_as_minus_one() {
        let rejected = Xmp {
            rating: super::super::REJECTED,
            ..Xmp::default()
        };

        let written = updated(None, &rejected);

        assert!(written.contains(r#"xmp:Rating="-1""#), "{written}");
        assert_eq!(read(&written).unwrap().rating, super::super::REJECTED);
    }

    #[test]
    fn clearing_a_label_removes_it_from_the_document() {
        let labelled = updated(
            None,
            &Xmp {
                label: Some("Red".to_string()),
                ..Xmp::default()
            },
        );

        let cleared = updated(Some(&labelled), &Xmp::default());

        assert!(!cleared.contains("xmp:Label"), "{cleared}");
        assert_eq!(read(&cleared).unwrap().label, None);
    }

    /// An unpicked frame should not leave digiKam's property in a sidecar that
    /// never had one.
    #[test]
    fn nothing_digikam_is_written_for_an_unpicked_frame() {
        let written = updated(None, &xmp(3, &[]));

        assert!(!written.contains("digiKam"), "{written}");
    }

    #[test]
    fn a_label_another_program_wrote_survives_a_round_trip() {
        let existing = format!(
            r#"<rdf:RDF xmlns:rdf="{NS_RDF}"><rdf:Description rdf:about=""
                 xmlns:xmp="{NS_XMP}" xmp:Label="Chartreuse"/></rdf:RDF>"#
        );

        let read_back = read(&existing).unwrap();
        assert_eq!(read_back.label.as_deref(), Some("Chartreuse"));

        let edited = updated(Some(&existing), &read_back);
        assert_eq!(read(&edited).unwrap().label.as_deref(), Some("Chartreuse"));
    }

    #[test]
    fn editing_twice_is_stable() {
        let once = updated(None, &xmp(3, &["A", "B"]));
        let twice = updated(Some(&once), &xmp(3, &["A", "B"]));

        assert_eq!(read(&once).unwrap(), read(&twice).unwrap());
    }
}

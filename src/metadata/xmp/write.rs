//! Writing a rating and keywords back into an XMP document.
//!
//! An existing document is edited rather than replaced: sidecars are routinely
//! shared with a raw converter that keeps its entire develop history in there,
//! and overwriting it would be destroying work.

use std::io::Cursor;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{NsReader, Writer};

use super::{namespace_of, Namespace, Xmp, NS_DC, NS_RDF, NS_XMP};

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
pub fn update(existing: Option<&str>, xmp: &Xmp) -> String {
    match existing.and_then(|document| edit(document, xmp)) {
        Some(edited) => edited,
        None => fresh(xmp),
    }
}

/// Rewrites an existing document, or gives up so a fresh one is written.
///
/// Gives up when the document has no `rdf:Description` to edit, which means it
/// is not XMP we can work with.
fn edit(document: &str, xmp: &Xmp) -> Option<String> {
    let mut reader = NsReader::from_str(document);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Our two properties are stripped wherever they appear and reinstated in
    // the first description, so a document cannot end up saying two things.
    let mut skipping = 0usize;
    let mut written = false;

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
            Event::Eof => break,
            Event::Start(ref element) | Event::Empty(ref element) => {
                let local = element.local_name();
                let name: &str = local.as_ref();

                if matches!(
                    (namespace, name),
                    (Namespace::Xmp, "Rating") | (Namespace::Dc, "subject")
                ) {
                    // An empty element has no subtree to skip.
                    if matches!(event, Event::Start(_)) {
                        skipping = 1;
                    }
                    continue;
                }

                if namespace == Namespace::Rdf && name == "Description" && !written {
                    written = true;
                    let element = element.to_owned();
                    let self_closing = matches!(event, Event::Empty(_));
                    write_description(&mut writer, &mut reader, &element, xmp, self_closing)
                        .ok()?;
                    continue;
                }

                writer.write_event(event).ok()?;
            }
            other => writer.write_event(other).ok()?,
        }
    }

    let edited = String::from_utf8(writer.into_inner().into_inner()).ok()?;

    written.then_some(edited)
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

    for attribute in element.attributes().flatten() {
        // Drop any rating already there, whatever prefix it used.
        let is_rating = {
            let (resolved, local) = reader.resolver_mut().resolve_attribute(attribute.key);
            let named_rating: bool = local.as_ref() == "Rating";
            namespace_of(&resolved) == Namespace::Xmp && named_rating
        };

        if is_rating {
            continue;
        }

        declares_xmp |= attribute.key.as_ref() == "xmlns:xmp";
        rebuilt.push_attribute(attribute);
    }

    // The `xmp` prefix has to be in scope for the attribute we are about to
    // add; declaring it again on this element is harmless if it already is.
    if !declares_xmp {
        rebuilt.push_attribute(("xmlns:xmp", NS_XMP));
    }

    let rating = xmp.rating.to_string();
    rebuilt.push_attribute(("xmp:Rating", rating.as_str()));

    writer.write_event(Event::Start(rebuilt))?;
    write_keywords(writer, &xmp.keywords)?;

    // A self-closing description has to become a pair now that it has content.
    if self_closing {
        writer.write_event(Event::End(BytesEnd::new(name)))?;
    }

    Ok(())
}

/// Writes `dc:subject` as an unordered bag, the form every reader understands.
fn write_keywords(writer: &mut Sink, keywords: &[String]) -> std::io::Result<()> {
    if keywords.is_empty() {
        return Ok(());
    }

    // Both elements carry their own namespace declaration so the block stays
    // valid wherever it is inserted.
    let mut subject = BytesStart::new("dc:subject");
    subject.push_attribute(("xmlns:dc", NS_DC));
    writer.write_event(Event::Start(subject))?;

    let mut bag = BytesStart::new("rdf:Bag");
    bag.push_attribute(("xmlns:rdf", NS_RDF));
    writer.write_event(Event::Start(bag))?;

    for keyword in keywords {
        writer.write_event(Event::Start(BytesStart::new("rdf:li")))?;
        writer.write_event(Event::Text(BytesText::new(keyword)))?;
        writer.write_event(Event::End(BytesEnd::new("rdf:li")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("rdf:Bag")))?;
    writer.write_event(Event::End(BytesEnd::new("dc:subject")))
}

/// A complete document, for a file that has no sidecar yet.
fn fresh(xmp: &Xmp) -> String {
    let items: String = xmp
        .keywords
        .iter()
        .map(|keyword| format!("     <rdf:li>{}</rdf:li>\n", escape(keyword)))
        .collect();

    let subject = if items.is_empty() {
        String::new()
    } else {
        format!("   <dc:subject>\n    <rdf:Bag>\n{items}    </rdf:Bag>\n   </dc:subject>\n")
    };

    format!(
        r#"<?xpacket begin="{bom}" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="{MARKER}">
 <rdf:RDF xmlns:rdf="{NS_RDF}">
  <rdf:Description rdf:about=""
    xmlns:xmp="{NS_XMP}"
    xmlns:dc="{NS_DC}"
   xmp:Rating="{rating}">
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

    fn xmp(rating: u8, keywords: &[&str]) -> Xmp {
        Xmp {
            rating,
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
        }
    }

    #[test]
    fn a_fresh_document_round_trips() {
        let written = update(None, &xmp(4, &["Slovakia", "Autumn"]));
        let back = read(&written).expect("reads back");

        assert_eq!(back.rating, 4);
        assert_eq!(back.keywords, vec!["Slovakia", "Autumn"]);
        assert!(written.contains(MARKER));
    }

    #[test]
    fn an_empty_annotation_still_produces_a_document() {
        let written = update(None, &Xmp::default());

        assert!(read(&written).expect("reads back").is_empty());
    }

    #[test]
    fn keywords_are_escaped() {
        let written = update(None, &xmp(0, &["black & white", "<odd>"]));

        assert_eq!(
            read(&written).unwrap().keywords,
            vec!["black & white", "<odd>"]
        );
    }

    #[test]
    fn editing_replaces_what_was_there() {
        let first = update(None, &xmp(1, &["Draft"]));
        let second = update(Some(&first), &xmp(5, &["Final", "Print"]));
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

        let edited = update(Some(&existing), &xmp(3, &["Keeper"]));
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

        let back = read(&update(Some(&existing), &xmp(2, &["Tagged"]))).unwrap();

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

        let edited = update(Some(&existing), &xmp(4, &[]));

        assert_eq!(read(&edited).unwrap().rating, 4);
        assert!(!edited.contains("<xmp:Rating>"));
    }

    #[test]
    fn something_that_is_not_xmp_is_replaced_wholesale() {
        let written = update(Some("this is not a document"), &xmp(3, &[]));

        assert_eq!(read(&written).unwrap().rating, 3);
        assert!(written.contains(MARKER));
    }

    #[test]
    fn clearing_a_rating_writes_a_zero_rather_than_forgetting() {
        let rated = update(None, &xmp(5, &["Keep"]));
        let cleared = update(Some(&rated), &Xmp::default());
        let back = read(&cleared).unwrap();

        assert_eq!(back.rating, 0);
        assert!(back.keywords.is_empty());
    }

    #[test]
    fn editing_twice_is_stable() {
        let once = update(None, &xmp(3, &["A", "B"]));
        let twice = update(Some(&once), &xmp(3, &["A", "B"]));

        assert_eq!(read(&once).unwrap(), read(&twice).unwrap());
    }
}

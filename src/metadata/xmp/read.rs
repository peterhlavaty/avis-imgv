//! Reading a rating and keywords out of an XMP packet.

use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::{NsReader, XmlVersion};

use super::{namespace_of, parse_pick, parse_rating, Namespace, Xmp};

/// Documents larger than this are not worth walking; a develop history can run
/// to megabytes and none of it concerns us past the first description.
const MAX_EVENTS: usize = 100_000;

/// Reads the rating and keywords from an XMP document.
///
/// Returns `None` when the text is not XMP at all. A document that parses but
/// carries neither property yields an empty [`Xmp`], which is different: it
/// means the file has been seen and has nothing set.
pub fn read(document: &str) -> Option<Xmp> {
    // Text is deliberately not trimmed: an entity reference splits a value
    // into several events, and trimming each would swallow the spaces around
    // it. Whole values are trimmed once they are assembled.
    let reader = &mut NsReader::from_str(document);

    let mut found = Xmp::default();
    let mut saw_rdf = false;
    // Which property's text we are currently collecting, if any, and what has
    // been collected so far. Text arrives in pieces because entity references
    // are separate events.
    let mut collecting: Option<Collecting> = None;
    let mut collected = String::new();

    for _ in 0..MAX_EVENTS {
        // The resolved namespace borrows the reader, so it is reduced to a
        // plain value before anything else touches the reader.
        let (namespace, event) = {
            let (resolved, event) = reader.read_resolved_event().ok()?;
            (namespace_of(&resolved), event)
        };

        match event {
            Event::Eof => break,
            Event::Start(ref element) | Event::Empty(ref element) => {
                let local = element.local_name();
                let name: &str = local.as_ref();

                // An empty element has no text and no closing tag, so starting
                // to collect at one would go on collecting for the rest of the
                // document: an empty `<dc:subject/>` used to turn every later
                // `rdf:li` — a hierarchical subject, a tone curve — into a
                // keyword.
                let opening = matches!(event, Event::Start(_));

                match (namespace, name) {
                    (Namespace::Rdf, "Description") => {
                        saw_rdf = true;
                        read_description_attributes(reader, element, &mut found);
                    }
                    (Namespace::Rdf, "li") => collected.clear(),
                    (Namespace::Rdf, _) => saw_rdf = true,
                    // Lightroom's hierarchical form, read into its own list:
                    // a program that understands paths reads this and one that
                    // does not still finds the leaves in `dc:subject`.
                    (Namespace::Lightroom, "hierarchicalSubject") if opening => {
                        collecting = Some(Collecting::Hierarchy);
                        collected.clear();
                    }
                    (Namespace::Dc, "subject") if opening => {
                        collecting = Some(Collecting::Keywords);
                        // Whatever the previous property left behind is not
                        // part of this one: a rating used to arrive glued to
                        // the front of the first keyword.
                        collected.clear();
                    }
                    _ => {
                        if let (true, Some(property)) = (opening, property_of(namespace, name)) {
                            collecting = Some(Collecting::Scalar(property));
                            collected.clear();
                        }
                    }
                }
            }
            Event::End(ref element) => {
                let local = element.local_name();
                let name: &str = local.as_ref();

                match (namespace, name) {
                    (Namespace::Rdf, "li") if collecting == Some(Collecting::Keywords) => {
                        push_keyword(&mut found.keywords, &collected);
                        collected.clear();
                    }
                    (Namespace::Rdf, "li") if collecting == Some(Collecting::Hierarchy) => {
                        push_keyword(&mut found.hierarchy, &collected);
                        collected.clear();
                    }
                    (Namespace::Lightroom, "hierarchicalSubject") => {
                        push_keyword(&mut found.hierarchy, &collected);
                        collected.clear();
                        collecting = None;
                    }
                    (Namespace::Dc, "subject") => {
                        // Some writers store a single keyword as plain text
                        // rather than a bag.
                        push_keyword(&mut found.keywords, &collected);
                        collected.clear();
                        collecting = None;
                    }
                    _ => {
                        if let Some(property) = property_of(namespace, name) {
                            apply(&mut found, property, &collected);
                            collected.clear();
                            collecting = None;
                        }
                    }
                }
            }
            Event::Text(ref text) if collecting.is_some() => {
                collected.push_str(&text.xml10_content());
            }
            // Entity references arrive as their own event, so a keyword such
            // as "black &amp; white" is assembled from three of them.
            Event::GeneralRef(ref reference) if collecting.is_some() => {
                collected.push_str(&resolve(reference));
            }
            _ => {}
        }
    }

    saw_rdf.then_some(found)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Collecting {
    Scalar(Property),
    Keywords,
    /// Lightroom's `hierarchicalSubject`, whose items are paths.
    Hierarchy,
}

/// The scalar properties may also appear as attributes of `rdf:Description`,
/// which is the compact form most writers use for them.
fn read_description_attributes(
    reader: &mut NsReader<&[u8]>,
    element: &BytesStart<'_>,
    found: &mut Xmp,
) {
    for attribute in element.attributes().flatten() {
        let property = {
            let (resolved, local) = reader.resolver_mut().resolve_attribute(attribute.key);
            property_of(namespace_of(&resolved), local.as_ref())
        };

        let Some(property) = property else {
            continue;
        };

        let Ok(value) = attribute.normalized_value(XmlVersion::Implicit1_0) else {
            continue;
        };

        apply(found, property, &value);
    }
}

/// The scalar properties the viewer maintains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Property {
    Rating,
    Label,
    Pick,
    Orientation,
}

/// Which of them a namespace and a local name is, if any.
fn property_of(namespace: Namespace, name: &str) -> Option<Property> {
    match (namespace, name) {
        (Namespace::Xmp, "Rating") => Some(Property::Rating),
        (Namespace::Xmp, "Label") => Some(Property::Label),
        (Namespace::DigiKam, "PickLabel") => Some(Property::Pick),
        (Namespace::Tiff, "Orientation") => Some(Property::Orientation),
        _ => None,
    }
}

/// Puts a property's text where it belongs.
fn apply(found: &mut Xmp, property: Property, text: &str) {
    match property {
        Property::Rating => {
            if let Some(rating) = parse_rating(text) {
                found.rating = rating;
            }
        }
        Property::Label => {
            let label = text.trim();
            found.label = (!label.is_empty()).then(|| label.to_string());
        }
        Property::Orientation => {
            if let Ok(value) = text.trim().parse::<u32>() {
                found.orientation = crate::metadata::Orientation::from_exif(value);
            }
        }
        Property::Pick => {
            if let Some(picked) = parse_pick(text) {
                found.picked = picked;
            }
        }
    }
}

/// Expands an entity reference: a numeric one, or one of the five XML defines.
fn resolve(reference: &BytesRef<'_>) -> String {
    if let Ok(Some(character)) = reference.resolve_char_ref() {
        return character.to_string();
    }

    match reference.xml10_content().as_ref() {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        // An entity from a document type declaration we have not read; the
        // reference itself is the most honest thing to keep.
        other => return format!("&{other};"),
    }
    .to_string()
}

fn push_keyword(keywords: &mut Vec<String>, text: &str) {
    let keyword = text.trim();

    if !keyword.is_empty() && !keywords.iter().any(|existing| existing == keyword) {
        keywords.push(keyword.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::super::{NS_DC, NS_RDF, NS_XMP};
    use super::*;

    /// Wraps properties in the surrounding boilerplate every XMP document has.
    fn document(description_attributes: &str, body: &str) -> String {
        format!(
            r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="{NS_RDF}">
  <rdf:Description rdf:about=""
    xmlns:xmp="{NS_XMP}"
    xmlns:dc="{NS_DC}"{description_attributes}>
{body}
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#
        )
    }

    #[test]
    fn reads_a_rating_written_as_an_attribute() {
        let xmp = read(&document(r#" xmp:Rating="4""#, "")).unwrap();

        assert_eq!(xmp.rating, 4);
        assert!(xmp.keywords.is_empty());
    }

    #[test]
    fn reads_a_rating_written_as_an_element() {
        let xmp = read(&document("", "<xmp:Rating>3</xmp:Rating>")).unwrap();
        assert_eq!(xmp.rating, 3);
    }

    #[test]
    fn reads_keywords_from_a_bag() {
        let body = r#"<dc:subject>
     <rdf:Bag>
      <rdf:li>Slovakia</rdf:li>
      <rdf:li>Autumn</rdf:li>
     </rdf:Bag>
    </dc:subject>"#;

        let xmp = read(&document("", body)).unwrap();
        assert_eq!(xmp.keywords, vec!["Slovakia", "Autumn"]);
    }

    #[test]
    fn reads_both_at_once() {
        let body = r#"<dc:subject><rdf:Seq><rdf:li>Macro</rdf:li></rdf:Seq></dc:subject>"#;
        let xmp = read(&document(r#" xmp:Rating="5""#, body)).unwrap();

        assert_eq!(xmp.rating, 5);
        assert_eq!(xmp.keywords, vec!["Macro"]);
    }

    #[test]
    fn unusual_prefixes_still_resolve() {
        let document = format!(
            r#"<meta xmlns:r="{NS_RDF}" xmlns:adobe="{NS_XMP}">
                 <r:RDF><r:Description adobe:Rating="2"/></r:RDF>
               </meta>"#
        );

        assert_eq!(read(&document).unwrap().rating, 2);
    }

    #[test]
    fn a_document_without_rdf_is_not_xmp() {
        assert_eq!(read("<html><body>not xmp</body></html>"), None);
        assert_eq!(read(""), None);
    }

    #[test]
    fn a_document_with_nothing_set_is_still_a_document() {
        assert!(read(&document("", "")).unwrap().is_empty());
    }

    #[test]
    fn malformed_xml_does_not_panic() {
        assert!(read("<rdf:RDF><unclosed").is_none());
        assert!(read("<<<>>>").is_none());
    }

    #[test]
    fn duplicate_keywords_are_collapsed() {
        let body = r#"<dc:subject><rdf:Bag>
            <rdf:li>Trees</rdf:li><rdf:li>Trees</rdf:li>
        </rdf:Bag></dc:subject>"#;

        assert_eq!(read(&document("", body)).unwrap().keywords, vec!["Trees"]);
    }

    #[test]
    fn escaped_keywords_come_back_decoded() {
        let body = "<dc:subject><rdf:Bag><rdf:li>black &amp; white</rdf:li></rdf:Bag></dc:subject>";

        assert_eq!(
            read(&document("", body)).unwrap().keywords,
            vec!["black & white"]
        );
    }

    #[test]
    fn text_outside_the_properties_is_ignored() {
        let body = "<xmp:CreatorTool>Some Editor</xmp:CreatorTool>";

        assert!(read(&document("", body)).unwrap().is_empty());
    }

    /// The rating's text used to be left in the buffer, so it arrived glued to
    /// the front of the next thing collected.
    #[test]
    fn a_rating_does_not_bleed_into_the_first_keyword() {
        let body = "<xmp:Rating>3</xmp:Rating><dc:subject>Macro</dc:subject>";
        let xmp = read(&document("", body)).unwrap();

        assert_eq!(xmp.rating, 3);
        assert_eq!(xmp.keywords, vec!["Macro"]);
    }

    /// An empty subject used to leave the reader collecting for the rest of
    /// the document, so a raw converter's own lists became keywords.
    #[test]
    fn an_empty_subject_does_not_swallow_the_rest_of_the_document() {
        let body = concat!(
            "<dc:subject/>",
            r#"<lr:hierarchicalSubject xmlns:lr="http://ns.adobe.com/lightroom/1.0/">"#,
            "<rdf:Bag><rdf:li>Places|Slovakia</rdf:li></rdf:Bag>",
            "</lr:hierarchicalSubject>",
        );

        assert!(read(&document("", body)).unwrap().keywords.is_empty());
    }

    /// The same, for a rating with no body.
    #[test]
    fn an_empty_rating_element_does_not_collect() {
        let body = concat!(
            "<xmp:Rating/>",
            "<crs:ToneCurve xmlns:crs=\"http://ns.adobe.com/camera-raw-settings/1.0/\">",
            "<rdf:Seq><rdf:li>0, 0</rdf:li></rdf:Seq>",
            "</crs:ToneCurve>",
        );

        let xmp = read(&document("", body)).unwrap();

        assert_eq!(xmp.rating, 0);
        assert!(xmp.keywords.is_empty());
    }
}

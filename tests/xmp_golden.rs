//! What the sidecar writer actually puts on disk, byte for byte.
//!
//! The unit tests check that what is written can be read back, which is the
//! property that matters most and is not the only one that matters. A sidecar
//! is read by Lightroom, darktable, digiKam and exiftool, and none of them is
//! going to be re-run against this test suite — so the shape of the document,
//! not only its meaning, is part of the contract.
//!
//! A round trip cannot see a change of shape at all: rename a namespace prefix
//! or move a rating from an attribute to an element and this viewer will still
//! read its own output perfectly while every other program stops seeing the
//! rating. These files are the record of what was agreed, and a diff against
//! them is the warning that something other than the tests is about to change.
//!
//! When a change here is deliberate, the fix is to look at the diff, satisfy
//! yourself that the new document is still what those programs expect, and
//! update the file.

use std::path::PathBuf;

use avis_imgv::metadata::xmp::{write, Xmp};

fn golden(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{name}.xmp"));

    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} could not be read: {e}", path.display()))
        // Written with unix line endings; git may hand them back with the
        // platform's own, and a line ending is not what this test is about.
        .replace("\r\n", "\n")
}

fn check(name: &str, xmp: &Xmp) {
    let written = write::update(None, xmp).expect("a fresh document is always written");

    assert_eq!(
        written.replace("\r\n", "\n"),
        golden(name),
        "the sidecar written for `{name}` is not the one on record; \
         see tests/golden/{name}.xmp"
    );
}

#[test]
fn a_bare_sidecar_matches_the_record() {
    check("bare", &Xmp::default());
}

#[test]
fn a_rating_matches_the_record() {
    check(
        "rated",
        &Xmp {
            rating: 4,
            ..Xmp::default()
        },
    );
}

/// A rejection is a rating of minus one, which is the convention every other
/// program reads.
#[test]
fn a_rejection_matches_the_record() {
    check(
        "rejected",
        &Xmp {
            rating: -1,
            ..Xmp::default()
        },
    );
}

/// The pick flag has no standard of its own, so it is written as digiKam's
/// `PickLabel` — and which namespace that lives in is exactly the sort of
/// thing a round trip cannot notice changing.
#[test]
fn a_pick_and_a_colour_label_match_the_record() {
    check(
        "picked-labelled",
        &Xmp {
            rating: 5,
            picked: true,
            label: Some("Red".to_string()),
            ..Xmp::default()
        },
    );
}

/// Keywords go in an `rdf:Bag` under `dc:subject`, and the characters XML
/// cares about are escaped.
#[test]
fn keywords_match_the_record() {
    check(
        "keywords",
        &Xmp {
            rating: 2,
            keywords: vec![
                "Tatras".to_string(),
                "Winter & Ice".to_string(),
                "<odd>".to_string(),
            ],
            ..Xmp::default()
        },
    );
}

/// A keyword filed under levels is written twice: the path in Lightroom's
/// `hierarchicalSubject`, and the leaf in `dc:subject` where every reader
/// looks. Which of the two a program reads is exactly what a round trip
/// cannot see.
#[test]
fn a_hierarchy_matches_the_record() {
    check(
        "hierarchy",
        &Xmp {
            keywords: vec!["Tatras".to_string(), "Vienna".to_string()],
            hierarchy: vec![
                "Places|Slovakia|Tatras".to_string(),
                "Places|Austria|Vienna".to_string(),
            ],
            ..Xmp::default()
        },
    );
}

/// And every one of them still reads back as what it was, so the record is a
/// record of something correct rather than only of something stable.
#[test]
fn every_record_reads_back_as_what_it_says() {
    let cases = [
        (
            "rated",
            Xmp {
                rating: 4,
                ..Xmp::default()
            },
        ),
        (
            "rejected",
            Xmp {
                rating: -1,
                ..Xmp::default()
            },
        ),
        (
            "picked-labelled",
            Xmp {
                rating: 5,
                picked: true,
                label: Some("Red".to_string()),
                ..Xmp::default()
            },
        ),
        (
            "keywords",
            Xmp {
                rating: 2,
                keywords: vec![
                    "Tatras".to_string(),
                    "Winter & Ice".to_string(),
                    "<odd>".to_string(),
                ],
                ..Xmp::default()
            },
        ),
        (
            "hierarchy",
            Xmp {
                keywords: vec!["Tatras".to_string(), "Vienna".to_string()],
                hierarchy: vec![
                    "Places|Slovakia|Tatras".to_string(),
                    "Places|Austria|Vienna".to_string(),
                ],
                ..Xmp::default()
            },
        ),
    ];

    for (name, expected) in cases {
        let read = avis_imgv::metadata::xmp::read::read(&golden(name));
        assert_eq!(read, Some(expected), "reading tests/golden/{name}.xmp back");
    }
}

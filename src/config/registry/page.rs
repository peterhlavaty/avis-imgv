//! Where each setting is drawn.
//!
//! The file is storage and the page is a view over it. The struct layout of
//! `Config` is a fine shape for JSON and a poor one for a person: it puts the
//! grey behind the photograph under `image_view` beside the cache radius, and
//! the rejects folder under `cull` where somebody looking for "delete" will not
//! find it. So a page is named for what somebody is doing, and a row lands on
//! it whatever section of the file holds it.
//!
//! Nothing is called General, Advanced, Miscellaneous or Other. Microsoft names
//! all four as labels to avoid, and the plain/advanced line cannot be drawn
//! honestly anyway: `raw.source` is the most consequential setting a raw
//! shooter has and would land in Advanced under any technical rule.

/// The eleven pages, in the order the navigation list shows them.
///
/// Ordered by how often a thing is wanted, not by the shape of the struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Page {
    OpeningAFolder,
    ThePhotograph,
    TheContactSheet,
    Marks,
    Keywords,
    MovingAndDeleting,
    RawFiles,
    Slideshow,
    SpeedAndMemory,
    TheWindow,
    KeysAndMouse,
}

impl Page {
    pub const ALL: &'static [Page] = &[
        Page::OpeningAFolder,
        Page::ThePhotograph,
        Page::TheContactSheet,
        Page::Marks,
        Page::Keywords,
        Page::MovingAndDeleting,
        Page::RawFiles,
        Page::Slideshow,
        Page::SpeedAndMemory,
        Page::TheWindow,
        Page::KeysAndMouse,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Page::OpeningAFolder => "Opening a folder",
            Page::ThePhotograph => "The photograph",
            Page::TheContactSheet => "The contact sheet",
            Page::Marks => "Stars, flags and labels",
            Page::Keywords => "Keywords",
            Page::MovingAndDeleting => "Moving and deleting",
            Page::RawFiles => "Raw files",
            Page::Slideshow => "Slideshow",
            Page::SpeedAndMemory => "Speed and memory",
            Page::TheWindow => "The window",
            Page::KeysAndMouse => "Keys and mouse",
        }
    }

    /// What the page is about, drawn under its heading.
    pub fn sentence(self) -> &'static str {
        match self {
            Page::OpeningAFolder => {
                "What the viewer starts with, what it shows, and in what order."
            }
            Page::ThePhotograph => "How one photograph is drawn, and what is drawn over it.",
            Page::TheContactSheet => "The grid of thumbnails, and the strip under the photograph.",
            Page::Marks => "Stars, flags and colour labels, and where they are written.",
            Page::Keywords => "The keyword list, the panel, and what is written to a sidecar.",
            Page::MovingAndDeleting => {
                "Where photographs are sent, what the bin means, and what is asked first."
            }
            Page::RawFiles => "How a raw file is turned into a picture, and what pairs with what.",
            Page::Slideshow => "How long each picture is held, and whether it moves.",
            Page::SpeedAndMemory => {
                "What the viewer holds in RAM and on the graphics card, and how hard it works."
            }
            Page::TheWindow => "How the interface itself looks, and what is remembered.",
            Page::KeysAndMouse => "Every key and every gesture the viewer reads.",
        }
    }
}

/// A block within a page.
///
/// Disclosure is inside a group on the page it belongs to, never a second page
/// for the advanced half: a page named for what a field is made of is a page
/// nobody can predict the contents of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    /// The rows drawn first, under no heading.
    Plain,
    /// What the viewer opens with.
    Starting,
    /// The order and the narrowing a new folder gets.
    Browsing,
    /// What is drawn over the picture.
    Overlay,
    /// The frame and the ground behind it.
    Framing,
    /// Movement: zoom, pan, a page.
    Movement,
    /// The cells themselves.
    Cells,
    /// The strip of thumbnails.
    Filmstrip,
    /// How runs of frames are detected.
    Grouping,
    /// Where things are sent.
    Destinations,
    /// What is asked before something irreversible.
    Confirmations,
    /// How a raw file is developed.
    Developing,
    /// What lives in RAM.
    Memory,
    /// What lives on the graphics card.
    Graphics,
    /// How many threads, and how much per frame.
    Work,
    /// Colours and text size.
    Appearance,
    /// The panels and their widths.
    Panels,
    /// The pointer.
    Mouse,
    /// The keyboard map.
    Keys,
    /// The context menus.
    Menus,
    /// Paths, the version, and the buttons under a separator.
    Footer,
}

impl Group {
    /// The heading, or nothing for the rows that need none.
    pub fn label(self) -> Option<&'static str> {
        Some(match self {
            Group::Plain => return None,
            Group::Starting => "What a launch starts with",
            Group::Browsing => "What a folder opens as",
            Group::Overlay => "What is drawn over the photograph",
            Group::Framing => "The frame, and the ground behind it",
            Group::Movement => "Zooming and panning",
            Group::Cells => "The cells",
            Group::Filmstrip => "The strip under the photograph",
            Group::Grouping => "What counts as one run of frames",
            Group::Destinations => "Where photographs go",
            Group::Confirmations => "What is asked first",
            Group::Developing => "Developing",
            Group::Memory => "What is held in RAM",
            Group::Graphics => "What is held on the graphics card",
            Group::Work => "How hard the viewer works",
            Group::Appearance => "How it looks",
            Group::Panels => "The panels",
            Group::Mouse => "The mouse",
            Group::Keys => "The keys",
            Group::Menus => "The menus",
            Group::Footer => "Files",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_is_named_and_explained() {
        for page in Page::ALL {
            assert!(!page.label().is_empty());
            assert!(!page.sentence().is_empty());
        }
    }

    /// Four names Microsoft asks for by name, and none of them is here.
    #[test]
    fn no_page_is_called_general_or_advanced() {
        for page in Page::ALL {
            let name = page.label().to_lowercase();
            for banned in ["general", "advanced", "miscellaneous", "other"] {
                assert_ne!(name, banned, "a page is called {banned}");
            }
        }
    }

    #[test]
    fn there_are_eleven_pages() {
        assert_eq!(Page::ALL.len(), 11);
    }
}

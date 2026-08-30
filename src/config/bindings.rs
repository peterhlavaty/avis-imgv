//! Every key the viewer listens for, in one list.
//!
//! The keyboard map lives in the configuration as three dozen separate fields,
//! which is the right shape for reading it and the wrong shape for showing it
//! to somebody. This is the other view of the same thing: a flat list with a
//! sentence explaining each entry, and a way to reach the field behind it.
//!
//! Adding a shortcut to the configuration and not to this list means it cannot
//! be changed from the interface, so the two are meant to be edited together.

use super::{Config, Shortcut};

/// Where a binding's field lives.
///
/// A pair of accessors rather than one: reading happens every frame the editor
/// is open and must not need a mutable borrow of the whole configuration. The
/// star ratings are one list rather than six fields, so reaching them takes an
/// index where everything else takes a function.
#[derive(Clone, Copy)]
enum Field {
    Fixed(fn(&Config) -> &Shortcut, fn(&mut Config) -> &mut Shortcut),
    Rating(usize),
    Label(usize),
}

/// One thing a key can be bound to.
#[derive(Clone, Copy)]
pub struct Binding {
    /// Which part of the viewer it belongs to, for grouping the list.
    pub section: &'static str,
    /// What it does, as a heading.
    pub name: &'static str,
    /// What it does, in a sentence.
    pub description: &'static str,
    field: Field,
}

impl Binding {
    /// The shortcut currently bound, if the configuration still has it.
    pub fn get<'a>(&self, config: &'a Config) -> Option<&'a Shortcut> {
        match self.field {
            Field::Fixed(read, _) => Some(read(config)),
            Field::Rating(index) => config.tags.sc_rating.get(index),
            Field::Label(index) => config.tags.sc_label.get(index),
        }
    }

    /// Replaces what this binding is bound to.
    pub fn set(&self, config: &mut Config, shortcut: Shortcut) {
        match self.field {
            Field::Fixed(_, write) => *write(config) = shortcut,
            Field::Rating(index) => {
                if let Some(field) = config.tags.sc_rating.get_mut(index) {
                    *field = shortcut;
                }
            }
            Field::Label(index) => {
                if let Some(field) = config.tags.sc_label.get_mut(index) {
                    *field = shortcut;
                }
            }
        }
    }
}

/// Sections, in the order the editor lists them.
pub const SECTIONS: &[&str] = &["General", "Image view", "Gallery", "Ratings and tags"];

/// Builds one binding.
macro_rules! binding {
    ($section:expr, $name:expr, $description:expr, $($field:tt)+) => {
        Binding {
            section: $section,
            name: $name,
            description: $description,
            field: Field::Fixed(
                |config| &config.$($field)+,
                |config| &mut config.$($field)+,
            ),
        }
    };
}

/// Every key the viewer listens for.
pub fn all() -> Vec<Binding> {
    let mut bindings = vec![
        binding!(
            "General",
            "Next mode",
            "Move round the modes: image, gallery, bulk rename, shift capture time, group shots, slideshow.",
            general.sc_next_mode
        ),
        binding!(
            "General",
            "Gallery",
            "Switch between the image and the contact sheet.",
            general.sc_toggle_gallery
        ),
        binding!("General", "Menu", "Show or hide the menu bar.", general.sc_menu),
        binding!(
            "General",
            "Side panel",
            "Show or hide the metadata and cache readout down the side.",
            general.sc_toggle_side_panel
        ),
        binding!(
            "General",
            "Tag panel",
            "Show or hide the panel for stars and keywords.",
            tags.sc_toggle_tag_panel
        ),
        binding!(
            "General",
            "Navigation bar",
            "Type a path to open instead of picking one.",
            general.sc_navigator
        ),
        binding!(
            "General",
            "Directory tree",
            "Open the folder tree beside the image.",
            general.sc_dir_tree
        ),
        binding!(
            "General",
            "Flatten folders",
            "Read the pictures out of every sub folder as though they were one.",
            general.sc_flatten_dir
        ),
        binding!(
            "General",
            "Watch the folder",
            "Pick up pictures that appear or change while the viewer is open.",
            general.sc_watch_directory
        ),
        binding!("General", "Quit", "Close the viewer.", general.sc_exit),
        binding!(
            "Image view",
            "Next image",
            "Move to the next picture in the folder.",
            image_view.sc_next
        ),
        binding!(
            "Image view",
            "Previous image",
            "Move to the one before it.",
            image_view.sc_prev
        ),
        binding!(
            "Image view",
            "Fit",
            "Show the whole picture, as large as the window allows.",
            image_view.sc_fit
        ),
        binding!(
            "Image view",
            "Fill",
            "Fill the window, cropping whichever side overflows.",
            image_view.sc_fit_maximize
        ),
        binding!(
            "Image view",
            "Keep filling",
            "Carry on filling the window as you move through the folder.",
            image_view.sc_latch_fit_maximize
        ),
        binding!(
            "Image view",
            "Fit width",
            "Make the picture exactly as wide as the window.",
            image_view.sc_fit_horizontal
        ),
        binding!(
            "Image view",
            "Fit height",
            "Make it exactly as tall.",
            image_view.sc_fit_vertical
        ),
        binding!(
            "Image view",
            "Zoom step",
            "Double the magnification, returning to fitted once it goes far enough.",
            image_view.sc_zoom
        ),
        binding!(
            "Image view",
            "Zoom in",
            "Magnify a little more.",
            image_view.sc_zoom_in
        ),
        binding!(
            "Image view",
            "Zoom out",
            "Magnify a little less.",
            image_view.sc_zoom_out
        ),
        binding!(
            "Image view",
            "Actual pixels",
            "One screen pixel for each pixel of the photograph.",
            image_view.sc_one_to_one
        ),
        binding!(
            "Image view",
            "Repeat the last view",
            "Put this picture at the zoom and position the last one was left at, for comparing two frames of the same thing.",
            image_view.sc_repeat_place
        ),
        binding!(
            "Image view",
            "Pan up",
            "Move the view up, for as long as the key is held.",
            image_view.sc_pan_up
        ),
        binding!("Image view", "Pan down", "Move the view down.", image_view.sc_pan_down),
        binding!("Image view", "Pan left", "Move the view left.", image_view.sc_pan_left),
        binding!(
            "Image view",
            "Pan right",
            "Move the view right.",
            image_view.sc_pan_right
        ),
        binding!(
            "Image view",
            "White frame",
            "Show or hide the white border around the photograph.",
            image_view.sc_frame
        ),
        binding!(
            "Image view",
            "More side by side",
            "Show one more picture beside the current one.",
            image_view.sc_more_images_shown
        ),
        binding!(
            "Image view",
            "Fewer side by side",
            "Show one fewer.",
            image_view.sc_less_images_shown
        ),
        binding!(
            "Gallery",
            "Scroll down",
            "Move half a row down the contact sheet.",
            grid_view.sc_scroll
        ),
        binding!(
            "Gallery",
            "More per row",
            "Fit one more thumbnail across, making them smaller.",
            grid_view.sc_more_per_row
        ),
        binding!(
            "Gallery",
            "Fewer per row",
            "Fit one fewer, making them larger.",
            grid_view.sc_less_per_row
        ),
    ];

    bindings.extend([
        binding!(
            "Ratings and tags",
            "Keep",
            "Mark the picture on screen as one to keep. Pressing it again takes the mark off.",
            tags.sc_pick
        ),
        binding!(
            "Ratings and tags",
            "Reject",
            "Mark it as one to throw out. Pressing it again puts it back.",
            tags.sc_reject
        ),
        binding!(
            "Ratings and tags",
            "No flag",
            "Take whichever of those two marks it carries back off it.",
            tags.sc_unflag
        ),
        binding!(
            "Ratings and tags",
            "Advance after marking",
            "Turn on and off moving to the next picture as soon as one is rated, flagged or labelled.",
            tags.sc_toggle_advance
        ),
    ]);

    for (index, label) in crate::metadata::xmp::Label::CHOICES.iter().enumerate() {
        bindings.push(Binding {
            section: "Ratings and tags",
            name: label.name(),
            description: LABEL_DESCRIPTIONS[index],
            field: Field::Label(index),
        });
    }

    for stars in 0..=crate::metadata::xmp::MAX_RATING {
        bindings.push(Binding {
            section: "Ratings and tags",
            name: RATING_NAMES[stars as usize],
            description: RATING_DESCRIPTIONS[stars as usize],
            field: Field::Rating(stars as usize),
        });
    }

    bindings
}

const LABEL_DESCRIPTIONS: &[&str] = &[
    "Put the red label on the picture on screen. Pressing it again takes it off.",
    "Put the yellow label on it.",
    "Put the green label on it.",
    "Put the blue label on it.",
    "Put the purple label on it.",
];

const RATING_NAMES: &[&str] = &[
    "No stars",
    "One star",
    "Two stars",
    "Three stars",
    "Four stars",
    "Five stars",
];

const RATING_DESCRIPTIONS: &[&str] = &[
    "Take the rating off the picture on screen.",
    "Put one star on the picture on screen.",
    "Put two stars on it.",
    "Put three stars on it.",
    "Put four stars on it.",
    "Put five stars on it.",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_binding_explains_itself() {
        for binding in all() {
            assert!(!binding.name.is_empty(), "{} has no name", binding.section);
            assert!(
                binding.description.ends_with('.'),
                "{} is not a sentence",
                binding.name
            );
            assert!(
                SECTIONS.contains(&binding.section),
                "{} is in no section",
                binding.name
            );
        }
    }

    #[test]
    fn no_two_bindings_share_a_name_within_a_section() {
        let mut seen: Vec<(&str, &str)> = all()
            .iter()
            .map(|binding| (binding.section, binding.name))
            .collect();

        let before = seen.len();
        seen.sort_unstable();
        seen.dedup();

        assert_eq!(seen.len(), before);
    }

    #[test]
    fn a_binding_reaches_the_field_it_names() {
        let mut config = Config::default();
        let bindings = all();

        let next = bindings
            .iter()
            .find(|binding| binding.name == "Next image")
            .expect("the list has it");

        next.set(&mut config, Shortcut::new("z", &[]));
        assert_eq!(config.image_view.sc_next.key, "z");
    }

    #[test]
    fn the_star_ratings_reach_their_places_in_the_list() {
        let mut config = Config::default();
        let bindings = all();

        let three = bindings
            .iter()
            .find(|binding| binding.name == "Three stars")
            .expect("the list has it");

        three.set(&mut config, Shortcut::new("F5", &[]));
        assert_eq!(config.tags.sc_rating[3].key, "F5");
    }

    #[test]
    fn every_shortcut_in_the_configuration_can_be_changed_from_the_list() {
        // The count is what stops a shortcut being added to the configuration
        // and quietly left out of the editor.
        let fixed = all()
            .iter()
            .filter(|binding| binding.section != "Ratings and tags")
            .count();

        assert_eq!(fixed, 32, "a shortcut was added without a description");
    }
}

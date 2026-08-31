//! The table itself, one file per section of the configuration file.
//!
//! Split by *section* rather than by page, because a section is where the field
//! lives and a page is only where it is drawn: a row moving from one page to
//! another is an edit to one line, and a row moving between files would not be.

use std::sync::OnceLock;

use super::access::{Access, Choice, List};
use super::effect::{Effect, Scope};
use super::page::{Group, Page};
use super::Row;

/// Builds one row.
///
/// The field path goes last so the accessor macros can take it as a token
/// stream, which is what lets `image_view.overlay_text_size` be written once
/// rather than four times.
macro_rules! row {
    // With a line saying why it has no control of its own.
    (
        $page:ident / $group:ident,
        $path:literal,
        $label:literal,
        $sentence:literal,
        [$($alias:literal),* $(,)?],
        $effect:ident,
        $scope:ident,
        $access:expr,
        explained: $explained:literal $(,)?
    ) => {
        Row {
            page: Page::$page,
            group: Group::$group,
            path: $path,
            label: $label,
            sentence: $sentence,
            aliases: &[$($alias),*],
            effect: Effect::$effect,
            scope: Scope::$scope,
            access: $access,
            explained: Some($explained),
        }
    };
    (
        $page:ident / $group:ident,
        $path:literal,
        $label:literal,
        $sentence:literal,
        [$($alias:literal),* $(,)?],
        $effect:ident,
        $scope:ident,
        $access:expr $(,)?
    ) => {
        Row {
            page: Page::$page,
            group: Group::$group,
            path: $path,
            label: $label,
            sentence: $sentence,
            aliases: &[$($alias),*],
            effect: Effect::$effect,
            scope: Scope::$scope,
            access: $access,
            explained: None,
        }
    };
}

macro_rules! boolean {
    ($($field:tt)+) => {
        Access::Bool(|c| c.$($field)+, |c, v| c.$($field)+ = v)
    };
}

macro_rules! whole {
    ($ty:ty, $min:expr, $max:expr, $unit:literal, $rail:expr, $($field:tt)+) => {
        Access::Int {
            get: |c| c.$($field)+ as i64,
            set: |c, v| c.$($field)+ = v as $ty,
            min: $min,
            max: $max,
            unit: $unit,
            rail: $rail,
        }
    };
}

macro_rules! decimal {
    ($min:expr, $max:expr, $unit:literal, $rail:expr, $($field:tt)+) => {
        Access::Float {
            get: |c| c.$($field)+,
            set: |c, v| c.$($field)+ = v,
            min: $min,
            max: $max,
            unit: $unit,
            rail: $rail,
        }
    };
}

macro_rules! text {
    ($($field:tt)+) => {
        Access::Text(|c| c.$($field)+.clone(), |c, v| c.$($field)+ = v)
    };
}

macro_rules! template {
    ($($field:tt)+) => {
        Access::Template(|c| c.$($field)+.clone(), |c, v| c.$($field)+ = v)
    };
}

macro_rules! optional_text {
    ($($field:tt)+) => {
        Access::Path(|c| c.$($field)+.clone(), |c, v| c.$($field)+ = v)
    };
}

macro_rules! key {
    ($($field:tt)+) => {
        Access::Key(|c| &c.$($field)+, |c| &mut c.$($field)+)
    };
}

mod added;
mod browsing;
mod cache;
mod cull;
mod fixed;
mod general;
mod grid_view;
mod image_view;
mod raw;
mod slideshow;
mod tags;

/// Every row, built once on the first ask.
///
/// A `OnceLock` rather than a `static`, because the rows generated per index —
/// the six ratings, the five colour labels, the user actions — cannot be
/// written out by hand, and because the alternative the editor had was a `Vec`
/// allocated on every frame it drew.
pub fn rows() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();

    ROWS.get_or_init(|| {
        let mut rows = Vec::with_capacity(180);

        rows.push(row!(
            TheWindow / Footer,
            "version",
            "File version",
            "Which build's conventions this file was written to. The one key here \
             nobody should ever change by hand: it is how the viewer knows whether a \
             default it has since moved still needs bringing forward.",
            ["migration", "brought forward"],
            None,
            None,
            Access::ReadOnly(|c| c.version.to_string()),
        ));

        rows.extend(general::rows());
        rows.extend(image_view::rows());
        rows.extend(grid_view::rows());
        rows.extend(tags::rows());
        rows.extend(cull::rows());
        rows.extend(raw::rows());
        rows.extend(slideshow::rows());
        rows.extend(cache::rows());
        rows.extend(browsing::rows());
        rows.extend(added::rows());
        rows.extend(fixed::rows());

        rows
    })
}

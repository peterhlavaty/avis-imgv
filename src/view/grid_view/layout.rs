//! Grid geometry: how large a cell is, how many fit, and which images they
//! hold.

use std::ops::Range;

/// A grid sized for the available width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub columns: usize,
    /// Width of one cell.
    pub cell: f32,
    /// Height of one cell: the picture, plus the strip the marks are drawn in.
    ///
    /// Cells used to be square, which for a folder of landscape photographs
    /// left a band of background above and below every row — about forty-four
    /// per cent of the contact sheet drawn in grey.
    pub row: f32,
    /// How much of `row` belongs to the picture rather than to the marks.
    pub picture: f32,
    pub rows: usize,
    /// Space left over, split either side to keep the grid centred.
    pub padding: f32,
}

impl Layout {
    /// Computes the layout for `total` images across `columns` columns.
    ///
    /// `aspect` is how wide a cell's picture is against its height: 1.5 is the
    /// three-to-two most cameras shoot. `caption` is the height of the strip
    /// under it, zero when nothing is being drawn there.
    pub fn new(
        available_width: f32,
        columns: usize,
        total: usize,
        aspect: f32,
        caption: f32,
    ) -> Layout {
        let columns = columns.max(1);
        let exact = (available_width / columns as f32).max(1.0);
        // Rounding down to a whole number of pixels keeps neighbouring cells
        // from disagreeing about where their shared edge is.
        let cell = exact.floor().max(1.0);

        let aspect = if aspect.is_finite() && aspect > 0.0 {
            aspect
        } else {
            1.0
        };

        let picture = (cell / aspect).floor().max(1.0);
        let caption = caption.max(0.0).floor();

        Layout {
            columns,
            cell,
            picture,
            row: picture + caption,
            rows: total.div_ceil(columns),
            padding: (available_width - cell * columns as f32).max(0.0) / 2.0,
        }
    }

    /// Images held by a range of rows, clamped to the collection.
    pub fn indices(&self, rows: Range<usize>, total: usize) -> Range<usize> {
        let start = (rows.start * self.columns).min(total);
        let end = (rows.end * self.columns).min(total);

        start..end.max(start)
    }

    pub fn row_of(&self, index: usize) -> usize {
        index / self.columns
    }

    /// Scroll offset that brings `index` to the top of the viewport.
    pub fn scroll_offset_of(&self, index: usize) -> f32 {
        self.row_of(index) as f32 * self.row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three columns of three-to-two pictures with no caption strip.
    fn layout(width: f32, columns: usize, total: usize) -> Layout {
        Layout::new(width, columns, total, 1.5, 0.0)
    }

    #[test]
    fn divides_the_width_into_whole_pixels() {
        let grid = layout(1000.0, 3, 10);

        assert_eq!(grid.columns, 3);
        assert_eq!(grid.cell, 333.0);
        assert_eq!(grid.padding, 0.5);
    }

    /// The point of the change: a row is as tall as the picture in it, not as
    /// wide as the cell.
    #[test]
    fn a_row_is_as_tall_as_the_picture_it_holds() {
        let grid = layout(900.0, 3, 10);

        assert_eq!(grid.cell, 300.0);
        assert_eq!(grid.picture, 200.0);
        assert_eq!(grid.row, 200.0);
    }

    #[test]
    fn the_caption_strip_makes_the_row_taller() {
        let grid = Layout::new(900.0, 3, 10, 1.5, 18.0);

        assert_eq!(grid.picture, 200.0);
        assert_eq!(grid.row, 218.0);
        assert_eq!(grid.scroll_offset_of(3), 218.0);
    }

    #[test]
    fn a_square_cell_is_still_available() {
        let grid = Layout::new(900.0, 3, 10, 1.0, 0.0);

        assert_eq!(grid.picture, grid.cell);
    }

    #[test]
    fn a_nonsense_aspect_does_not_produce_a_nonsense_cell() {
        for aspect in [0.0, -2.0, f32::NAN, f32::INFINITY] {
            let grid = Layout::new(900.0, 3, 10, aspect, 0.0);
            assert!(grid.picture >= 1.0, "{aspect}");
            assert!(grid.row.is_finite(), "{aspect}");
        }
    }

    #[test]
    fn counts_a_partial_last_row() {
        assert_eq!(layout(900.0, 3, 10).rows, 4);
        assert_eq!(layout(900.0, 3, 9).rows, 3);
        assert_eq!(layout(900.0, 3, 0).rows, 0);
    }

    #[test]
    fn maps_rows_to_images() {
        let grid = layout(900.0, 3, 10);

        assert_eq!(grid.indices(0..2, 10), 0..6);
        // The last row is short.
        assert_eq!(grid.indices(3..4, 10), 9..10);
        // Rows past the end hold nothing.
        assert_eq!(grid.indices(10..12, 10), 10..10);
    }

    #[test]
    fn maps_images_back_to_rows() {
        let grid = layout(900.0, 3, 10);

        assert_eq!(grid.row_of(0), 0);
        assert_eq!(grid.row_of(2), 0);
        assert_eq!(grid.row_of(3), 1);
        assert_eq!(grid.scroll_offset_of(3), grid.row);
    }

    #[test]
    fn a_zero_width_panel_still_produces_usable_cells() {
        let grid = layout(0.0, 5, 10);

        assert!(grid.cell >= 1.0);
        assert!(grid.row >= 1.0);
        assert_eq!(grid.columns, 5);
    }

    #[test]
    fn zero_columns_are_treated_as_one() {
        assert_eq!(layout(500.0, 0, 3).columns, 1);
    }
}

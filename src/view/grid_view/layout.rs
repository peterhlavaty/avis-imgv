//! Grid geometry: how many square cells fit, and which images they hold.

use std::ops::Range;

/// A grid sized for the available width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub columns: usize,
    /// Side of one square cell.
    pub cell: f32,
    pub rows: usize,
    /// Space left over, split either side to keep the grid centred.
    pub padding: f32,
}

impl Layout {
    /// Computes the layout for `total` images across `columns` columns.
    pub fn new(available_width: f32, columns: usize, total: usize) -> Layout {
        let columns = columns.max(1);
        let exact = (available_width / columns as f32).max(1.0);
        // Rounding down to a whole number of pixels keeps neighbouring cells
        // from disagreeing about where their shared edge is.
        let cell = exact.floor().max(1.0);

        Layout {
            columns,
            cell,
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
        self.row_of(index) as f32 * self.cell
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divides_the_width_into_whole_pixels() {
        let layout = Layout::new(1000.0, 3, 10);

        assert_eq!(layout.columns, 3);
        assert_eq!(layout.cell, 333.0);
        assert_eq!(layout.padding, 0.5);
    }

    #[test]
    fn counts_a_partial_last_row() {
        assert_eq!(Layout::new(900.0, 3, 10).rows, 4);
        assert_eq!(Layout::new(900.0, 3, 9).rows, 3);
        assert_eq!(Layout::new(900.0, 3, 0).rows, 0);
    }

    #[test]
    fn maps_rows_to_images() {
        let layout = Layout::new(900.0, 3, 10);

        assert_eq!(layout.indices(0..2, 10), 0..6);
        // The last row is short.
        assert_eq!(layout.indices(3..4, 10), 9..10);
        // Rows past the end hold nothing.
        assert_eq!(layout.indices(10..12, 10), 10..10);
    }

    #[test]
    fn maps_images_back_to_rows() {
        let layout = Layout::new(900.0, 3, 10);

        assert_eq!(layout.row_of(0), 0);
        assert_eq!(layout.row_of(2), 0);
        assert_eq!(layout.row_of(3), 1);
        assert_eq!(layout.scroll_offset_of(3), layout.cell);
    }

    #[test]
    fn a_zero_width_panel_still_produces_usable_cells() {
        let layout = Layout::new(0.0, 5, 10);

        assert!(layout.cell >= 1.0);
        assert_eq!(layout.columns, 5);
    }

    #[test]
    fn zero_columns_are_treated_as_one() {
        assert_eq!(Layout::new(500.0, 0, 3).columns, 1);
    }
}

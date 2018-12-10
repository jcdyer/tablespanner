//! Core functionality: Uses strongly typed objects to represent input and
//! output. Public interfaces will wrap this code with common types for ease
//! of use.

use std::collections::HashMap;
use std::hash::Hash;

/// Represents the number of rows and columns occupied by a given table cell.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Span {
    rows: usize,
    cols: usize,
}

impl Span {
    /// Create a new Span with the specified dimensions.
    ///
    /// # Panics:
    ///
    /// This constructor panics if the value provided for rows or cols is zero.
    pub(crate) fn new(rows: usize, cols: usize) -> Span {
        if rows == 0 {
            panic!("Error constructing Span. Zero value provided for Span.rows.")
        } else if cols == 0 {
            panic!("Error constructing Span. Zero value provided for Span.cols.")
        }
        Span { rows, cols }
    }
}

impl Default for Span {

    /// Construct a default Span.
    ///
    /// A derived Default trait would construct a value of
    /// `Span { rows: 0, cols: 0 }` so we have to implement this manually.
    fn default() -> Span {
        Span::new(1, 1)
    }
}

/// Type alias for input table data, without span information.
pub(crate) type TableSpec<T> = Vec<Vec<T>>;

/// Type alias for output table layout with spanned cells rendered as `None`.
pub(crate) type TableLayout<T> = Vec<Vec<Option<T>>>;


/// [PRIVATE] Tracks which columns are currently occupied by active row
/// spans.
///
/// # Note:
///
/// * This object is completely unaware of multi-column spans.  The caller
///   is responsible for tracking all columns of a multi-column span by
///   calling `RowSpanTracker::track(..)` on each column separately.
///
/// * This object does not track which rows or columns belong to which
///   spans, only that they are spanned by *some* cell.
///
/// * This version maintains state, and tracks row spans relative to the
///   current row.  An alternative implementation that tracks all spans
///   statelessly might be more flexible, but would require the caller to
///   track information like current row.
struct RowSpanTracker(HashMap<usize, usize>);

impl RowSpanTracker {
    /// Create an empty RowSpanTracker object
    fn new() -> RowSpanTracker {
        RowSpanTracker(HashMap::new())
    }

    /// Track a new rowspan for the given column.  Caller should provide
    /// the total number of spanned rows for the column.
    fn track(&mut self, col_index: usize, row_count: usize) {
        if row_count > 1 {
            self.0.insert(col_index, row_count);
        }
    }

    /// Decrement all the active spans.  This should be called after each
    /// row is fully processed.
    fn dec(&mut self) {
        let keys = self.0.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            if let Some(value) = self.0.get_mut(&key) {
                if *value > 1 {
                    *value -= 1;
                } else {
                    self.0.remove(&key);
                }
            }
        }
    }

    /// Report if the current column is part of an active rowspan.
    fn spanned(&self, col_index: usize) -> bool {
        self.0.get(&col_index).unwrap_or(&0) > &0
    }

    /// Report the highest column that is part of an active rowspan.
    ///
    /// If there are no active rowspans, returns None.
    ///
    fn max_spanned(&self) -> Option<usize> {
        self.0.keys().max().cloned()
    }
}

/// [PRIVATE] Given a candidate column, and the cell's column count, return
/// `true` if the cell can be fit into this location of the table.
fn cell_fits(col: usize, col_count: usize, active_row_spans: &RowSpanTracker) -> bool {
    for peek in col..col + col_count {
        if active_row_spans.spanned(peek) {
            return false;
        }
    }
    true
}

/// Determine the layout of table cells given the available spans and the
/// data for the table.
pub(crate) fn layout_table<T>(spaninfo: &HashMap<T, Span>, data: &TableSpec<T>) -> TableLayout<T>
where
    T: Hash + Eq + Clone,
{
    let mut table: TableLayout<T> = Vec::new();
    let mut active_row_spans = RowSpanTracker::new();
    for inrow in data {
        let mut row = Vec::new();
        for cell in inrow.iter() {
            let span = spaninfo.get(&cell).cloned().unwrap_or_default();

            while !cell_fits(row.len(), span.cols, &active_row_spans) {
                row.push(None);
            }

            active_row_spans.track(row.len(), span.rows);
            row.push(Some(cell.clone()));
            for _ in 1..span.cols {
                active_row_spans.track(row.len(), span.rows);
                row.push(None);
            }
        }
        table.push(row);
        active_row_spans.dec();
    }

    // Handle trailing spanned rows.
    while let Some(col) = active_row_spans.max_spanned() {
        table.push(vec![None; col + 1]);
        active_row_spans.dec();
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;
    /// If no span specifications are given,
    /// return a simple table that matches the input
    #[test]
    fn basic_table_layout() {
        let spanspec = HashMap::new();
        let data = vec![
            vec!["A", "B", "C"],
            vec!["D", "E", "F"],
            vec!["G", "H", "I"],
        ];
        let expected = vec![
            vec![Some("A"), Some("B"), Some("C")],
            vec![Some("D"), Some("E"), Some("F")],
            vec![Some("G"), Some("H"), Some("I")],
        ];
        let result = layout_table(&spanspec, &data);
        assert_eq!(result, expected);
    }

    #[test]
    fn layout_with_colspan() {
        let mut spanspec = HashMap::new();
        spanspec.insert("D", Span::new(1, 2));
        let data = vec![vec!["A", "B", "C"], vec!["D", "E"], vec!["G", "H", "I"]];
        let expected = vec![
            vec![Some("A"), Some("B"), Some("C")],
            vec![Some("D"), None, Some("E")],
            vec![Some("G"), Some("H"), Some("I")],
        ];
        let result = layout_table(&spanspec, &data);
        assert_eq!(result, expected);
    }

    #[test]
    fn layout_with_rowspan() {
        let mut spanspec = HashMap::new();
        spanspec.insert("E", Span::new(2, 1));
        let data = vec![
            vec!["A", "B", "C"],
            vec!["D", "E", "F"],
            vec!["G", "H"],
            vec!["J", "K", "L"],
        ];
        let expected = vec![
            vec![Some("A"), Some("B"), Some("C")],
            vec![Some("D"), Some("E"), Some("F")],
            vec![Some("G"), None, Some("H")],
            vec![Some("J"), Some("K"), Some("L")],
        ];
        let result = layout_table(&spanspec, &data);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_block_span() {
        let mut spanspec = HashMap::new();
        spanspec.insert("D", Span::new(3, 2));
        spanspec.insert("E", Span::new(1, 2));
        let data = vec![
            vec!["A", "B", "C"],
            vec!["D", "E", "F"],
            vec!["G", "H", "I"],
            vec!["J", "K", "L"],
            vec!["M", "N", "O"],
        ];
        let expected = vec![
            vec![Some("A"), Some("B"), Some("C")],
            vec![Some("D"), None, Some("E"), None, Some("F")],
            vec![None, None, Some("G"), Some("H"), Some("I")],
            vec![None, None, Some("J"), Some("K"), Some("L")],
            vec![Some("M"), Some("N"), Some("O")],
        ];
        let result = layout_table(&spanspec, &data);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_overlapping_row_spans() {
        let mut spanspec = HashMap::new();
        spanspec.insert("E", Span::new(2, 1));
        spanspec.insert("H", Span::new(2, 1));
        let data = vec![
            vec!["A", "B", "C"],
            vec!["D", "E", "F"],
            vec!["G", "H", "I"],
            vec!["J", "K", "L"],
            vec!["M", "N", "O"],
        ];
        let expected = vec![
            vec![Some("A"), Some("B"), Some("C")],
            vec![Some("D"), Some("E"), Some("F")],
            vec![Some("G"), None, Some("H"), Some("I")],
            vec![Some("J"), Some("K"), None, Some("L")],
            vec![Some("M"), Some("N"), Some("O")],
        ];
        let result = layout_table(&spanspec, &data);
        assert_eq!(result, expected);
    }

    #[test]
    fn rowspan_blocking_colspans() {
        // If a rowspan blocks a colspan, push the colspan back past the end of the
        // rowspanned object.
        let mut spanspec = HashMap::new();
        spanspec.insert("E", Span::new(2, 1));
        spanspec.insert("G", Span::new(2, 2));
        let data = vec![
            vec!["A", "B", "C"],
            vec!["D", "E", "F"],
            vec!["G", "H", "I"],
            vec!["J", "K", "L"],
            vec!["M", "N", "O"],
        ];
        let expected = vec![
            vec![Some("A"), Some("B"), Some("C")],
            vec![Some("D"), Some("E"), Some("F")],
            vec![None, None, Some("G"), None, Some("H"), Some("I")],
            vec![Some("J"), Some("K"), None, None, Some("L")],
            vec![Some("M"), Some("N"), Some("O")],
        ];
        let result = layout_table(&spanspec, &data);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_trailing_rowspans() {
        let mut spanspec = HashMap::new();
        spanspec.insert("B", Span::new(3, 1));
        spanspec.insert("C", Span::new(2, 1));
        let data = vec![vec!["A", "B", "C"]];
        let expected = vec![
            vec![Some("A"), Some("B"), Some("C")],
            vec![None, None, None],
            vec![None, None],
        ];
        let result = layout_table(&spanspec, &data);
        assert_eq!(result, expected);
    }

}

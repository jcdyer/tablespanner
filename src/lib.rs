use self::engine::TableLayout;

/// Core functionality: Uses strongly typed objects to represent input and
/// output.  External Interfaces will wrap this code.
mod engine {
    use std::collections::HashMap;
    use std::hash::Hash;

    /// Represent the number of rows and columns occupied by a given table cell.
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
        /// The value for rows and cols must be non-zero.
        pub(crate) fn new(rows: usize, cols: usize) -> Span {
            if rows == 0 {
                panic!("Error constructing Span.  Zero value provided for Span.rows")
            } else if cols == 0 {
                panic!("Error constructing Span.  Zero value provided for Span.cols")
            }
            Span { rows, cols }
        }

        pub(crate) fn from_pair(pair: (usize, usize)) -> Span {
            Span::new(pair.0, pair.1)
        }
    }

    impl Default for Span {
        /// Construct a default Span.
        ///
        /// A derived Default trait would return a value of `Span { rows: 0, cols: 0 }`
        /// so we have to implement this manually.
        fn default() -> Span {
            Span::new(1, 1)
        }
    }

    pub(crate) type TableSpec<T> = Vec<Vec<T>>;
    pub(crate) type TableLayout<T> = Vec<Vec<Option<T>>>;

    struct ActiveRowSpans(HashMap<usize, usize>);

    impl ActiveRowSpans {
        fn new() -> ActiveRowSpans {
            ActiveRowSpans(HashMap::new())
        }

        /// Track a new rowspan for the given 0-indexed column.
        fn track(&mut self, col_index: usize, row_count: usize) {
            if row_count > 1 {
                self.0.insert(col_index, row_count);
            }
        }

        /// Decrement all the active spans.
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

        /// Report if the current column is part of an active rowspan
        fn spanned(&mut self, col_index: usize) -> bool {
            self.0.get(&col_index).unwrap_or(&0) > &0
        }

    }

    /// Determine the layout of table cells given the available spans and the
    /// data for the table
    pub(crate) fn layout_table<T>(
        spaninfo: &HashMap<T, Span>,
        data: &TableSpec<T>,
    ) -> TableLayout<T>
    where
        T: Hash + Eq + Clone,
    {
        let mut table: TableLayout<T> = Vec::new();
        let mut active_row_spans = ActiveRowSpans::new();
        for inrow in data {
            let mut row = Vec::new();
            let mut col = 0;
            for cell in inrow.iter() {
                while active_row_spans.spanned(col) {
                    row.push(None);
                    col += 1;
                }

                let span = spaninfo.get(&cell).cloned().unwrap_or_default();

                row.push(Some(cell.clone()));
                active_row_spans.track(col, span.rows);
                col += 1;
                for offset in 1..span.cols {
                    active_row_spans.track(col, span.rows);
                    row.push(None);
                    col += 1;
                }
            }
            table.push(row);
            active_row_spans.dec();
        }
        table

        /*

        data.iter()
            .map(|row| {
                row.iter()
                    .map(|cell| Some(cell.clone()))
                    .collect()
            })
            .collect()
            */
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
        fn test_blocked_col_spans() {
            let mut spanspec = HashMap::new();
            spanspec.insert("E", Span::new(2, 1));
            spanspec.insert("G", Span::new(1, 2));
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
                vec![None, None, Some("G"), Some("H"), Some("I")],  // The first None is because I can't fit a 2 colG in there.
                vec![Some("J"), Some("K"), None, Some("L")],
                vec![Some("M"), Some("N"), Some("O")],
            ];
            let result = layout_table(&spanspec, &data);
            assert_eq!(result, expected);
        }
    }
}

pub fn render_to_json<T>(_layout: TableLayout<T>) -> String {
    unimplemented!()
}

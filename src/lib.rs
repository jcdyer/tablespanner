//! # Tablespanner
//!
//! Implemented using the principles of Hexagonal Architecture: At the core is
//! the `engine` module, which deals with well-typed input data, and produces
//! well-typed output.  All input and output handling, public interfaces, and
//! type conversion happen in thin wrappers around the core.  This makes it
//! easy to add new wrappers for different front-ends.
//!
//! Rust was chosen as an implementation for its strong type system, and its
//! robust FFI capabilities that would allow it to be embedded in a phone app,
//! in a web front-end by compiling to WASM, or used either directly or in
//! another language on the server-side.
//!
//! ## CLI usage:
//!
//! ```sh
//! $ cargo build
//! $ target/debug/tablespanner '{"a": [2, 2]}' '[["a", "b"], ["c", "d"]]'
//! [["a", null, "b"], [null, null, "c", "d"]]
//! ```
//!
//! ## Library usage:
//!
//! ```rust
//! use std::collections::HashMap;
//! use tablespanner;
//!
//! fn render_table() -> Vec<Vec<Option<&'static str>>> {
//!     let mut spaninfo = HashMap::new();
//!     spaninfo.insert("a", (2, 2));
//!     let table = vec![
//!         vec!["a", "b"],
//!         vec!["c", "d"],
//!     ];
//!     tablespanner::render_table(spaninfo, table)
//! }
//! ```

use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::hash::Hash;
use self::engine::{Span, TableLayout};


mod engine;


/// IntoIterator where Item=(x, y) will work with a HashMap, a BTreeMap, or a Vec.
///
/// # Panics
///
/// This panics if any of the usize values passed to `Span::new` are zero.
fn simpledata_to_spaninfo<S, T>(data: S) -> HashMap<T, Span>
where
    S: IntoIterator<Item = (T, (usize, usize))>,
    T: Clone + Eq + Hash,
{
    data.into_iter()
        .map(|(cell, (rows, cols))| (cell, Span::new(rows, cols)))
        .collect()
}

/// Convert incoming JSON data to a HashMap<String, Span> or return an Error.
///
/// # Panics
///
/// This panics if any of the extracted usize values are zero.
fn json_to_spaninfo(data: &str) -> Result<HashMap<String, Span>, serde_json::Error> {
    // Given a type hint, serde_json will unpack well-typed JSON data into the
    // data structure we want.
    let hashmap: HashMap<String, (usize, usize)> = serde_json::from_str(data)?;
    Ok(simpledata_to_spaninfo(hashmap))
}

/// A TableSpec is already a simple data structure.
/// Serde can serialize it directly.
fn json_to_tablespec(data: &str) -> Result<engine::TableSpec<String>, serde_json::Error> {
    serde_json::from_str(data)
}

/// Serde will convert Option::None values to `null`
fn layout_to_json<T: Serialize>(layout: &TableLayout<T>) -> Result<String, serde_json::Error> {
    serde_json::to_string(layout)
}

/// Given a information about cell spans, and the input data for the table,
/// calculate the layout for the table, including spans.
///
/// Span info will often be provided as a `HashMap<T, (usize, usize)>` or
/// `BTreeMap<T (usize, usize)>`, but can be any data structure that implements
/// the appropriate `IntoIterator` trait.
///
/// The table spec is provided as a 2D `Vec` of cell identifiers.
///
/// Returns a 2D `Vec` of `Option<T>`, where cells that contain data are
/// returned as `Some(T)`, while cells that are spanned from other cells are
/// returned as `None`.
pub fn render_table<T, S>(spaninfo: S, tablespec: Vec<Vec<T>>) -> Vec<Vec<Option<T>>>
where
    S: IntoIterator<Item = (T, (usize, usize))>,
    T: Hash + Eq + Clone,
{
    let spaninfo = simpledata_to_spaninfo(spaninfo);
    engine::layout_table(&spaninfo, &tablespec)
}

/// Taking a JSON str representing the span info and another representing a
/// table spec, render the table as JSON output including spanned cells.
pub fn render_json_table(spaninfo: &str, tablespec: &str) -> Result<String, serde_json::Error> {
    let spaninfo = json_to_spaninfo(spaninfo)?;
    let tablespec = json_to_tablespec(tablespec)?;
    layout_to_json(&engine::layout_table(&spaninfo, &tablespec))
}

#[cfg(test)]
mod tests {
    use super::render_json_table;
    #[test]
    fn end_to_end() {
        let spaninfo = r#"{"B": [2, 2], "H": [1, 2]}"#;
        let tablespec = r#"[["A", "B", "C"], ["D", "E"], ["F", "G", "H"]]"#;
        assert_eq!(
            render_json_table(spaninfo, tablespec).unwrap(),
            r#"[["A","B",null,"C"],["D",null,null,"E"],["F","G","H",null]]"#
        );
    }
}

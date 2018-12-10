use clap::{App, Arg};

use tablespanner::render_json_table;

fn main() {
    let opts = App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Calculates JSON table layouts")
        .arg(Arg::with_name("SPANINFO")
             .help(r#"
                 JSON object mapping cell identifiers to [rows, cols], where
                 those values specify the number of cells the current cell spans.

                 Example: {"A": [1, 2], "E": [3, 3]}

                 Note: It is an error to pass zero for these values.
             "#)
             .required(true)
             .index(1))
        .arg(Arg::with_name("TABLESPEC")
             .help(r#"
                 A two-dimensional JSON array of cell identifiers for the table.

                 Example: [["A", "B"], ["C", "D"]]

                 Note: It is an error to pass non-string values, including
                 nulls.
             "#)
             .required(true)
             .index(2))
        .get_matches();
    let spaninfo = opts.value_of("SPANINFO").unwrap();
    let tablespec = opts.value_of("TABLESPEC").unwrap();
    println!("{}", render_json_table(spaninfo, tablespec).unwrap());
}

## Installation 

1.  Install Rust according to the instructions at https://rustup.rs/
2.  Download this repo: git clone https://github.com/jcdyer/tablespanner/
3.  ```bash
    cd tablespanner`
    ```
4.  To run tests:
    ```bash
    cargo test`
    ```
5.  To see generated documentation:
    ```bash
    cargo doc --open --document-private-items` to see documentation.
    ```
6.  To run from the command line:
    ```bash
    cargo run -- '{"A": [2, 1]}' '[["A", "B"], ["C, "D"]]'
    ```

    or:

    ```bash
    cargo build
    target/debug/tablespanner '{"A": [2, 1]}' '[["A", "B"], ["C, "D"]]'
    ```

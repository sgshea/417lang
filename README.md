# CSC417 Language Project
Language implemented in [Rust](https://www.rust-lang.org/)

## Files
- `Cargo.toml` Defines dependencies
- `src/main.rs` Program entry point, handles arguments
- `src/repl.rs` Reading from standard input
- `src/types.rs` Language type definitions and interpret function

### Tests
Some files (`types.rs`) have tests which can be run using `cargo test`

## Dependencies
`serde` and `serde_json`: JSON parsing
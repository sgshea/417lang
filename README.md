# CSC417 Language Project
Language implemented in [Rust](https://www.rust-lang.org/)

## Arguments
Program can be built and ran using `cargo run` or `cargo run --release` (release build is faster execution)

Default behavior (checkpoint 1) is to only take in a single string.
Passing in the `repl` argument will keep the program running until stopped.
> `cargo run --release -- --repl`

## Files
- `Cargo.toml` Defines dependencies
- `src/main.rs` Program entry point, handles arguments
- `src/repl.rs` Reading from standard input
- `src/types.rs` Language type definitions and interpret function

### Tests
Some files (`types.rs`) have tests which can be run using `cargo test`

## Dependencies
`serde` and `serde_json`: JSON parsing

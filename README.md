# CSC417 Language Project
Language implemented in [Rust](https://www.rust-lang.org/)

## Repository Structure
Repo organized into a Cargo Workspace, with two subcrates: `parser` and `interpreter`
- `parser` contains a Rust implementation of the parser, and is a library crate (not executable)
- `interpreter` is the Rust implementation of the language interpreter and is a binary crate

## Running
`run.sh` is a shortcut to `cargo run -p interpreter --release` which runs the interpreter crate.
- Input from stdin can be piped into the program, expecting a JSON AST

### Running Custom Parser
The custom parser is integrated into the interpreter using the `"parser"` feature.

The following command runs the interpreter with this feature, allowing it to take in the raw program.
`cargo run -p interpreter --release --features "parser"`
- Can use `run.sh -p` as a shortcut

## Dependencies
`serde` and `serde_json`: JSON parsing

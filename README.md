# CSC417 Language Project
Language implemented in [Rust](https://www.rust-lang.org/)
- This was a project created for a class "Programming Language Theory".
- We were required to implement an interpreter for a simple expression based language.
    - We could choose our implementation language
    - A class parser (in C) was provided which output a JSON AST

## Repository Structure
Repo organized into a Cargo Workspace, with two subcrates: `parser` and `interpreter`
- `parser` contains a Rust implementation of the parser, and is a library crate (not executable)
- `interpreter` is the Rust implementation of the language interpreter and is a binary crate
    - `index.html` and `index.js` in this folder are for the web builds (otherwise ignore)

## Running
`run.sh` is a shortcut to `cargo run -p interpreter --release` which runs the interpreter crate.
- Input from stdin can be piped into the program, expecting a JSON AST

### Running Custom Parser
The custom parser is integrated into the interpreter using the `"parser"` feature.

The following command runs the interpreter with this feature, allowing it to take in the raw program.
`cargo run -p interpreter --release --features "parser"`
- Can use `run.sh -a` as a shortcut

The parser can be run standalone using `run.sh -p`
## Dependencies
`serde` and `serde_json`: JSON parsing

## WASM
Commands to build for wasm (specific to my machine for now)
- Requires: [wasm-pack](https://github.com/rustwasm/wasm-pack)
- Navigate to `interpreter` package
    - Compiling this package and it pulls the parser as a dependency
    - Run the following command, using wasm-pack (wherever it was installed)
        - Using "wasm" feature which includes the dependencies needed (`wasm-bindgen` and `parser`)
- The files then need to be hosted on a local server (such as VSCode live preview or any simple server)
```
~/.cargo/bin/wasm-pack build --target web --features "wasm"
```
**GitHub Pages** at https://sgshea.github.io/417lang/
- Serves compiled wasm build from the `pages` branch
    - May not always be up to date
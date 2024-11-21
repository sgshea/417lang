use error::InterpError;
use interpreter::{Expr, Interpreter};

// WASM dependencies and functions locked behind "wasm" feature so that the crate does not need to be downloaded on normal runs
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

mod environment;
mod error;
mod functions;
mod interpreter;

/// Interprets a string, assumed to be valid JSON input from a parser
/// Returns either the interpreted expression or an error
pub fn interpret_string(
    input: &str,
    lexical_scope: bool,
    store_output: bool,
) -> Result<(Expr, Interpreter), InterpError> {
    // Interpret input
    match serde_json::from_str(input) {
        Err(_) => Err(InterpError::ParseError {
            message: "Unable to parse JSON into interpreter.".to_string(),
        }),
        Ok(j) => interpret_default(j, lexical_scope, store_output),
    }
}

/// Interprets the input JSON with the default environment
/// Returns either an error or a tuple of the resulting expression and the resulting environment
pub fn interpret_default(
    val: serde_json::Value,
    lexical_scope: bool,
    store_output: bool,
) -> Result<(Expr, Interpreter), InterpError> {
    let mut env = Interpreter::new(lexical_scope, store_output);
    Ok((Expr::eval(&val, &mut env)?, env))
}

/// Interprets a string
/// Returns the result of interpreting in string form
/// Expects the string to be valid JSON input
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn interpret_to_string(input: &str, lexical_scope: bool) -> String {
    match interpret_string(input, lexical_scope, true) {
        Err(e) => return e.to_string(),
        Ok((expr, env)) => {
            let output = env.global.output.concat();
            output + &expr.to_string()
        }
    }
}

/// Parses a string
/// Returns the result of parsing in string form
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_to_string(input: &str) -> String {
    use parser::parse;
    match parse("input", input) {
        Err(e) => return format!("{:?}", e.as_diagnostic()),
        Ok(ast) => return serde_json::to_string_pretty(&ast).unwrap(),
    }
}

/// Parses and then interprets a string
/// Returns the result of parsing and interpreting in string form
/// Same as above but exported for WASM
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn interpret_with_parser_to_string(input: &str, lexical_scope: bool) -> String {
    use parser::parse;
    match parse("input", input) {
        Err(e) => return format!("{:?}", e.as_diagnostic()),
        Ok(ast) => match interpret_default(ast, lexical_scope, true) {
            Err(e) => return e.to_string(),
            Ok((expr, env)) => {
                // Output the resulting expression after any stored output
                let output = env.global.output.concat();
                output + &expr.to_string()
            }
        },
    }
}

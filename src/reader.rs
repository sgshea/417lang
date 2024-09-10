//! Reads all of stdin as JSON

use std::io;

use serde_json::Value;

use crate::interpreter::InterpError;

pub fn read_stdin() -> Result<Value, InterpError> {
    match serde_json::from_reader(io::stdin()) {
        Err(_) => Err(InterpError::ParseError("Invalid JSON AST".to_string())),
        Ok(ok) => Ok(ok),
    }
}
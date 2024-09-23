//! Reads all of stdin as JSON

use std::io;

use serde_json::Value;

use crate::interpreter::InterpError;

pub fn read_stdin() -> Result<Value, InterpError> {
    match serde_json::from_reader(io::stdin()) {
        Err(e) => {
            // We don't want to error on empty input
            if e.is_eof() {
                Ok(Value::String("".to_string()))
            } else {
                Err(InterpError::ParseError(e.to_string()))
            }
        }
        Ok(ok) => Ok(ok),
    }
}
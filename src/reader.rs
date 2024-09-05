//! Reads all of stdin as JSON

use std::io;

use serde_json::Value;

pub fn read_stdin() -> Result<Value, serde_json::Error> {
    serde_json::from_reader(io::stdin())
}
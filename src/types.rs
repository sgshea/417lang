use std::{fmt, io};

use serde_json::Value;

/// All the types of the lispy language
pub enum LType {
    Integer(i64),
}

impl LType {
    /// Try to interpret a given string as a LType
    pub fn interpret_as_value(str: &str) -> Result<LType, LError> {
        let json: Result<Value, serde_json::Error> = serde_json::from_str(str);

        match json {
            Ok(val) => {
                match val {
                    Value::Number(ref num) => {
                        match num.as_i64() {
                            Some(n) => {
                                return Ok(LType::Integer(n))
                            }
                            None => {
                                return Err(LError::ParseError)
                            },
                        };
                    },
                    _ => {
                        Err(LError::ParseError)
                    },
                }
            },
            Err(_e) => {
                Err(LError::ParseError)
            }
        }
    }
}

impl fmt::Display for LType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LType::Integer(val) => write!(f, "{}", val)
        }
    }
}

/// Error types for all errors we may encounter in the interpreter
/// Prefer wrapping already existing error types
#[derive(Debug)]
pub enum LError {
    // Error while parsing
    ParseError,
    // Error while trying to read from IO
    IOError(io::Error)
}

impl fmt::Display for LError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LError::ParseError => write!(f, "error while parsing"),
            LError::IOError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for LError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LError::ParseError => None,
            LError::IOError(e) => Some(e),
        }
    }
}

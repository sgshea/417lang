use std::{fmt, io};

use serde_json::Value;

/// All the types of the language
#[derive(PartialEq, Eq, Debug)]
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
                                return Err(LError::ParseError(format!("{} is not a valid i64!", num)))
                            },
                        };
                    },
                    _ => {
                        Err(LError::ParseError(format!("{} is not an implemented type!", val)))
                    },
                }
            },
            Err(e) => {
                Err(LError::ParseError(format!("JSON parsing error: {}", e)))
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
    ParseError(String),
    // Error while trying to read from IO
    IOError(io::Error)
}

impl fmt::Display for LError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LError::ParseError(m) => write!(f, "Value Error: {}", m),
            LError::IOError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for LError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LError::ParseError(_) => None,
            LError::IOError(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_integer() -> Result<(), LError> {
        assert_eq!(LType::Integer(10), LType::interpret_as_value("10")?);
        assert_eq!(LType::Integer(-10), LType::interpret_as_value("-10")?);
        assert_eq!(LType::Integer(0), LType::interpret_as_value("0")?);

        Ok(())
    }

    #[test]
    fn parse_invalid_integer() -> Result<(), LError> {
        assert!(LType::interpret_as_value("\"\"").is_err_and(|e| matches!(e, LError::ParseError(_))));
        assert!(LType::interpret_as_value("str").is_err_and(|e| matches!(e, LError::ParseError(_))));
        assert!(LType::interpret_as_value("0.5").is_err_and(|e| matches!(e, LError::ParseError(_))));
        assert!(LType::interpret_as_value("-0.5").is_err_and(|e| matches!(e, LError::ParseError(_))));
        assert!(LType::interpret_as_value("[something]").is_err_and(|e| matches!(e, LError::ParseError(_))));
        assert!(LType::interpret_as_value("()").is_err_and(|e| matches!(e, LError::ParseError(_))));

        // Construct numbers larger and smaller than the range supported
        let big_num = i64::MAX as u64 + 10;
        // (Creating a string version manually less than i64::MIN)
        let small_num = "-".to_string() + &big_num.to_string();
        assert!(LType::interpret_as_value(&big_num.to_string()).is_err_and(|e| matches!(e, LError::ParseError(_))));
        assert!(LType::interpret_as_value(&small_num).is_err_and(|e| matches!(e, LError::ParseError(_))));

        Ok(())
    }
}

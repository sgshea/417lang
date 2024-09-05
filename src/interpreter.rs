use std::{fmt, io};

use serde_json::Value;

use crate::{environment::Environment, functions::Function};

/// All the types of the language
#[derive(PartialEq, Eq, Debug)]
pub enum Expr {
    // Integer value
    Integer(i64),
    // String value
    String(String),
    // Symbol
    Symbol(String),
    // Function
    Function(Function)
}

impl Expr {
    pub fn eval(val: serde_json::Value, env: &mut Environment) -> Result<Expr, LError> {
        match val {
            Value::Number(ref num) => {
                match num.as_i64() {
                    Some(n) => {
                        return Ok(Expr::Integer(n))
                    }
                    None => {
                        return Err(LError::ParseError(format!("{} is not a valid i64!", num)))
                    },
                };
            },
            Value::String(string) => {
                return Ok(Expr::String(string))
            },
            _ => {
                Err(LError::ParseError(format!("{} is not an implemented type! It is of JSON type {:?}", val, val)))
            },
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Integer(val) => write!(fmt, "{}", val),
            Expr::String(val) => write!(fmt, "{}", val),
            Expr::Symbol(val) => write!(fmt, "{}", val),
            Expr::Function(func) => {
                match func {
                    Function::LFunc(f) => write!(fmt, "{}", f),
                    Function::RFunc(f) => write!(fmt, "{:?}", f),
                }
            }
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

pub fn interpret(val: serde_json::Value, env: &mut Environment) -> Result<(), LError> {
    match Expr::eval(val, env) {
        Err(e) => Err(e),
        Ok(res) => {
            println!("{}", res);
            Ok(())
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parse_valid_integer() -> Result<(), LError> {
//         assert_eq!(Expr::Integer(10), Expr::interpret_as_value("10")?);
//         assert_eq!(Expr::Integer(-10), Expr::interpret_as_value("-10")?);
//         assert_eq!(Expr::Integer(0), Expr::interpret_as_value("0")?);

//         Ok(())
//     }

//     #[test]
//     fn parse_invalid_integer() -> Result<(), LError> {
//         // Construct numbers larger and smaller than the range supported
//         let big_num = i64::MAX as u64 + 10;
//         // (Creating a string version manually less than i64::MIN)
//         let small_num = "-".to_string() + &big_num.to_string();
//         assert!(Expr::interpret_as_value(&big_num.to_string()).is_err_and(|e| matches!(e, LError::ParseError(_))));
//         assert!(Expr::interpret_as_value(&small_num).is_err_and(|e| matches!(e, LError::ParseError(_))));

//         Ok(())
//     }

//     #[test]
//     fn parse_valid_string() -> Result<(), LError> {
//         // JSON parsing requires quotes, must include within test strings using escape characters
//         assert_eq!(Expr::String("test".to_string()), Expr::interpret_as_value("\"test\"")?);

//         Ok(())
//     }
// }

use std::{error::Error, fmt};

/// Error types for all errors we may encounter in the interpreter
#[derive(Debug)]
pub enum InterpError {
    // General parsing error
    ParseError {
        message: String,
    },
    // Argument error with function name and argument mismatch
    ArgumentError {
        func: String,
        expected: usize, // Expected number of arguments
        got: usize // Got this amount of arguments
    },
    // Symbol undefined such as when searching for identifier
    UndefinedError {
        symbol: String
    },
    // Type error for when a type is incorrect
    TypeError {
        expected: String,
        found: String,
    },
    // Error for general runtime errors (ex: divide by zero)
    RuntimeError {
        message: String,
    },
}

impl fmt::Display for InterpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpError::ParseError { message} => {
                write!(f, "Parse error: {}", message)
            },
            InterpError::ArgumentError { func, expected, got } => {
                write!(f, "Incorrect number of arguments supplied to function '{}': expected {}, got {}", func, expected, got)
            },
            InterpError::UndefinedError { symbol } => {
                write!(f, "Undefined symbol '{}'", symbol)
            }
            InterpError::TypeError { expected, found } => {
                write!(f, "Type error: expected {}, found {}", expected, found)
            }
            InterpError::RuntimeError { message } => {
                write!(f, "Runtime error: {}", message)
            }
        }
    }
}

impl Error for InterpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
use std::fmt::{self};

use serde_json::Value;

use crate::{environment::Environment, functions::Function};

/// All the types of the language
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Expr {
    // Integer value
    Integer(i64),
    // String value
    String(String),
    // Symbol value
    Symbol(String),
    // List of Expr
    List(Vec<Expr>),
    // Function
    Function(Function),
}

impl Expr {
    pub fn eval(val: &serde_json::Value, env: &mut Environment) -> Result<Expr, InterpError> {
        match val {
            Value::Number(ref num) => {
                // Turn a JSON number into an Expr::Integer
                match num.as_i64() {
                    Some(n) => return Ok(Expr::Integer(n)),
                    None => {
                        return Err(InterpError::ParseError(format!(
                            "{} is not a valid i64!",
                            num
                        )))
                    }
                };
            }
            Value::String(string) => {
                // Turn a string into a Expr::String
                return Ok(Expr::String(string.to_string()));
            }
            Value::Object(obj) => {
                if let Some(binding) = obj.get("Identifier") {
                    if let Some(bound) =
                        env.lookup(binding.as_str().expect("Identifier should be string"))
                    {
                        return Ok(bound);
                    } else {
                        return Ok(Expr::Symbol(
                            binding
                                .as_str()
                                .expect("Identifier should be string")
                                .to_string(),
                        ));
                    }
                } else if let Some(key) = obj.get("Block") {
                    // Processes block
                    Expr::eval(key, env)
                } else if let Some(arr) = obj.get("Application") {
                    match Expr::eval(arr, env)? {
                        Expr::List(list) => {
                            let (first, rest) =
                                list.split_first().ok_or(InterpError::ArgumentError)?;
                            match first {
                                Expr::Function(func) => match func {
                                    Function::RFunc { name: _name, func } => func(rest),
                                },
                                _ => Err(InterpError::ArgumentError),
                            }
                        }
                        _ => Err(InterpError::ArgumentError),
                    }
                } else if let Some(arr) = obj.get("Def") {
                    match Expr::eval(arr, env)? {
                        Expr::List(list) => {
                            let (first, rest) =
                                list.split_first().ok_or(InterpError::ArgumentError)?;
                            match first {
                                Expr::Symbol(symbol) => {
                                    // Bind to a list if the rest is multiple, else to the only other expr
                                    if rest.len() > 1 {
                                        env.bind(
                                            &symbol,
                                            Expr::List(rest.into_iter().cloned().collect()),
                                        )
                                    } else {
                                        env.bind(
                                            &symbol,
                                            rest.last().expect("Should only be one").clone(),
                                        )
                                    }
                                }
                                _ => Err(InterpError::ArgumentError),
                            }
                        }
                        _ => Err(InterpError::ArgumentError),
                    }
                } else {
                    Err(InterpError::ParseError(format!(
                        "Found JSON Object but not a known binding"
                    )))
                }
            }
            Value::Array(arr) => {
                // Convert a vector of JSON values into an Expr::List
                Ok(Expr::List(
                    arr.into_iter()
                        .map(|val| Expr::eval(val, env))
                        .collect::<Result<Vec<Expr>, InterpError>>()?,
                ))
            }
            _ => Err(InterpError::ParseError(format!(
                "{} is not an implemented type! It is of JSON type {:?}",
                val, val
            ))),
        }
    }

    pub fn into_i64(&self) -> Result<i64, InterpError> {
        match self {
            Expr::Integer(int) => Ok(*int),
            _ => Err(InterpError::ParseError(
                "Expression expected to be integer".to_string(),
            )),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Integer(val) => write!(fmt, "{}", val),
            Expr::String(val) => write!(fmt, "{}", val),
            Expr::Symbol(val) => write!(fmt, "{}", val),
            Expr::List(list) => write!(fmt, "{:#?}", list),
            Expr::Function(func) => match func {
                Function::RFunc { name, func: _ } => write!(fmt, "function: {}", name),
            },
        }
    }
}

/// Error types for all errors we may encounter in the interpreter
/// Prefer wrapping already existing error types
#[derive(Debug)]
pub enum InterpError {
    // Error while parsing x
    ParseError(String),
    // Error calling a function with wrong arguments
    ArgumentError,
}

impl fmt::Display for InterpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpError::ParseError(m) => write!(f, "Value Error while parsing {}", m),
            InterpError::ArgumentError => write!(f, "Argument Error"),
        }
    }
}

impl std::error::Error for InterpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            _ => None,
        }
    }
}

pub fn exprs_into_i64(exprs: &[Expr]) -> Result<Vec<i64>, InterpError> {
    exprs
        .into_iter()
        .map(|expr| expr.into_i64())
        .collect::<Result<Vec<i64>, InterpError>>()
}

pub fn interpret(val: serde_json::Value, env: &mut Environment) -> Result<(), InterpError> {
    match Expr::eval(&val, env) {
        Err(e) => Err(e),
        Ok(res) => {
            println!("{}", res);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_integer() -> Result<(), InterpError> {
        let mut env = Environment::default_environment();
        assert_eq!(Expr::Integer(12), Expr::eval(&serde_json::from_str("12").unwrap(), &mut env)?);
        assert_eq!(Expr::Integer(-500), Expr::eval(&serde_json::from_str("-500").unwrap(), &mut env)?);

        Ok(())
    }

    #[test]
    fn parse_invalid_integer() -> Result<(), InterpError> {
        let mut env = Environment::default_environment();
        // Construct numbers larger and smaller than the range supported
        let big_num = i64::MAX as u64 + 10;
        // (Creating a string version manually less than i64::MIN)
        let small_num = "-".to_string() + &big_num.to_string();
        assert!(Expr::eval(&serde_json::from_str(&big_num.to_string()).unwrap(), &mut env).is_err_and(|e| matches!(e, InterpError::ParseError(_))));
        assert!(Expr::eval(&serde_json::from_str(&small_num.to_string()).unwrap(), &mut env).is_err_and(|e| matches!(e, InterpError::ParseError(_))));

        Ok(())
    }

    #[test]
    fn parse_valid_string() -> Result<(), InterpError> {
        let mut env = Environment::default_environment();
        assert_eq!(Expr::String("rust".to_string()), Expr::eval(&serde_json::from_str("\"rust\"").unwrap(), &mut env)?);
        assert_eq!(Expr::String("🦀".to_string()), Expr::eval(&serde_json::from_str("\"🦀\"").unwrap(), &mut env)?);

        Ok(())
    }
}

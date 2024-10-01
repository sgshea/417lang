use std::fmt;

use serde_json::Value;

use crate::{environment::Environment, error::InterpError, functions::Function};

/// All the types of the language
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Expr {
    // Integer value
    Integer(i64),
    // Boolean
    Boolean(bool), // true, false
    // String value
    String(String),
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
                        return Err(InterpError::ParseError {
                            message: format!("{} is not a valid i64.", num),
                        });
                    }
                };
            }
            Value::Bool(bool) => Ok(Expr::Boolean(*bool)),
            Value::String(string) => {
                // Turn a string into a Expr::String
                return Ok(Expr::String(string.to_string()));
            }
            Value::Object(obj) => {
                if let Some(binding) = obj.get("Identifier") {
                    if let Value::String(str) = binding {
                        if let Some(val) = env.lookup(str) {
                            Ok(val)
                        } else {
                            Err(InterpError::UndefinedError {
                                symbol: str.to_string(),
                            })
                        }
                    } else {
                        Err(InterpError::ParseError {
                            message: "Expected string to lookup identifier.".to_string(),
                        })
                    }
                } else if let Some(key) = obj.get("Block") {
                    // Processes block
                    // Last expression is the return value
                    if let Expr::List(list) = Expr::eval(key, env)? {
                        // Return false for empty block
                        if let Some(last) = list.last() {
                            Ok(last.clone())
                        } else {
                            Ok(Expr::Boolean(false))
                        }
                    } else {
                        Err(InterpError::ParseError {
                            message: "Expected expressions within block.".to_string(),
                        })
                    }
                } else if let Some(arr) = obj.get("Application") {
                    if let Expr::List(list) = Expr::eval(arr, env)? {
                        let (first, rest) = list.split_first().ok_or(InterpError::ParseError {
                            message: "Function application on nothing.".to_string(),
                        })?;
                        if let Expr::Function(func) = first {
                            match func {
                                Function::RFunc { name: _name, func } => return func(rest),
                                Function::UFunc {
                                    name: _name,
                                    args,
                                    func,
                                } => {
                                    // Create a local environment, binding arguments
                                    // Pair together the rest slice to args
                                    env.bind(
                                        args.into_iter()
                                            .zip(rest.into_iter())
                                            .collect::<Vec<(&String, &Expr)>>(),
                                    );
                                    let res = Self::eval(func, env);
                                    env.pop_top_env();
                                    return res;
                                }
                            }
                        } else {
                            return Err(InterpError::TypeError {
                                expected: "function".to_string(),
                                found: first.to_string(),
                            });
                        }
                    }
                    Err(InterpError::ParseError {
                        message: "Expected function and arguments.".to_string(),
                    })
                } else if let Some(arr) = obj.get("Cond") {
                    if let Value::Array(arr) = arr {
                        // Expect "Clause"
                        // Returns the result of the first expression where it's condition was true
                        for statement in arr {
                            if let Value::Array(clause) =
                                statement.get("Clause").expect("Expect \"Clause\"")
                            {
                                // Splits the condition and expression away
                                let [condition, expr] = clause.as_slice() else {
                                    return Err(InterpError::ParseError { message: "Clause did not contain both a condition and expression.".to_string() });
                                };
                                // Store condition result
                                let condition = Expr::eval(condition, env)?;
                                // If it is a boolean that is true, we evaluate the expression
                                if let Expr::Boolean(b) = condition {
                                    if b {
                                        return Expr::eval(expr, env);
                                    }
                                } else {
                                    return Err(InterpError::TypeError {
                                        expected: "bool".to_string(),
                                        found: condition.to_string(),
                                    });
                                }
                                continue;
                            }
                        }
                    }

                    Ok(Expr::Boolean(false))
                } else if let Some(_arr) = obj.get("Def") {
                    todo!()
                } else {
                    Err(InterpError::ParseError { message: format!("Found JSON Object in AST but it does not contain a known keyword: {:?}", obj) })
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
            _ => Err(InterpError::ParseError {
                message: format!(
                    "{} is not an implemented type! It is of JSON type {:?}",
                    val, val
                ),
            }),
        }
    }

    pub fn into_i64(&self) -> Result<i64, InterpError> {
        match self {
            Expr::Integer(int) => Ok(*int),
            _ => Err(InterpError::TypeError {
                expected: "integer".to_string(),
                found: self.to_string(),
            }),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Integer(val) => write!(fmt, "{}", val),
            Expr::Boolean(val) => write!(fmt, "{}", val),
            Expr::String(val) => write!(fmt, "{}", val),
            Expr::List(list) => write!(fmt, "{:#?}", list),
            Expr::Function(func) => match func {
                Function::RFunc { name, func: _ } => write!(fmt, "function: {}", name),
                Function::UFunc {
                    name,
                    args: _,
                    func: _,
                } => write!(fmt, "function: {}", name),
            },
        }
    }
}

pub fn exprs_into_i64(exprs: &[Expr]) -> Result<Vec<i64>, InterpError> {
    exprs
        .into_iter()
        .map(|expr| expr.into_i64())
        .collect::<Result<Vec<i64>, InterpError>>()
}

pub fn interpret(val: serde_json::Value, env: &mut Environment) -> Result<Expr, InterpError> {
    Expr::eval(&val, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_integer() -> Result<(), InterpError> {
        let mut env = Environment::default_environment();
        assert_eq!(
            Expr::Integer(12),
            Expr::eval(&serde_json::from_str("12").unwrap(), &mut env)?
        );
        assert_eq!(
            Expr::Integer(-500),
            Expr::eval(&serde_json::from_str("-500").unwrap(), &mut env)?
        );

        Ok(())
    }

    #[test]
    fn parse_invalid_integer() -> Result<(), InterpError> {
        let mut env = Environment::default_environment();
        // Construct numbers larger and smaller than the range supported
        let big_num = i64::MAX as u64 + 10;
        // (Creating a string version manually less than i64::MIN)
        let small_num = "-".to_string() + &big_num.to_string();
        assert!(Expr::eval(
            &serde_json::from_str(&big_num.to_string()).unwrap(),
            &mut env
        )
        .is_err_and(|e| matches!(e, InterpError::ParseError { message: _ })));
        assert!(Expr::eval(
            &serde_json::from_str(&small_num.to_string()).unwrap(),
            &mut env
        )
        .is_err_and(|e| matches!(e, InterpError::ParseError { message: _ })));

        Ok(())
    }

    #[test]
    fn parse_valid_string() -> Result<(), InterpError> {
        let mut env = Environment::default_environment();
        assert_eq!(
            Expr::String("rust".to_string()),
            Expr::eval(&serde_json::from_str("\"rust\"").unwrap(), &mut env)?
        );
        assert_eq!(
            Expr::String("🦀".to_string()),
            Expr::eval(&serde_json::from_str("\"🦀\"").unwrap(), &mut env)?
        );

        Ok(())
    }
}

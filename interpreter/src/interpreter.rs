use std::fmt;

use serde_json::{Map, Value};

use crate::{
    environment::Environment,
    error::InterpError,
    functions::{function_application, parse_anonymous_function, Function},
};

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
            Value::Number(num) => num
                .as_i64()
                .ok_or_else(|| InterpError::TypeError {
                    expected: "i64".to_string(),
                    found: num.to_string(),
                })
                .map(|i| Expr::Integer(i)),
            Value::Bool(bool) => Ok(Expr::Boolean(*bool)),
            Value::String(string) => {
                return Ok(Expr::String(string.to_string()));
            }
            Value::Array(arr) => Ok(Expr::List(
                arr.into_iter()
                    .map(|val| Expr::eval(val, env))
                    .collect::<Result<Vec<Expr>, InterpError>>()?,
            )),
            Value::Object(obj) => parse_object(obj, env),
            _ => Err(InterpError::ParseError {
                message: format!(
                    "{} is not an implemented type! It is of JSON type {:?}",
                    val, val
                ),
            }),
        }
    }
}

impl TryInto<bool> for &Expr {
    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Expr::Boolean(bool) => Ok(*bool),
            _ => Err(InterpError::TypeError {
                expected: "bool".to_string(),
                found: self.to_string(),
            }),
        }
    }

    type Error = InterpError;
}

impl TryInto<i64> for &Expr {
    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            Expr::Integer(int) => Ok(*int),
            _ => Err(InterpError::TypeError {
                expected: "integer".to_string(),
                found: self.to_string(),
            }),
        }
    }

    type Error = InterpError;
}

impl TryInto<String> for Expr {
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Expr::String(str) => Ok(str.to_string()),
            _ => Err(InterpError::TypeError {
                expected: "string".to_string(),
                found: self.to_string(),
            }),
        }
    }

    type Error = InterpError;
}

impl TryInto<String> for &Expr {
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Expr::String(str) => Ok(str.to_string()),
            _ => Err(InterpError::TypeError {
                expected: "string".to_string(),
                found: self.to_string(),
            }),
        }
    }

    type Error = InterpError;
}

/// Parse a JSON object, looking for the keys that correspond to certain behaviors
fn parse_object(obj: &Map<String, Value>, env: &mut Environment) -> Result<Expr, InterpError> {
    // First see if there is an identifier
    if let Some(binding) = obj.get("Identifier").and_then(|val| val.as_str()) {
        return env
            .lookup(binding)
            .ok_or_else(|| InterpError::UndefinedError {
                symbol: binding.to_string(),
            });
    }

    // Handle blocks
    if let Some(key) = obj.get("Block") {
        return parse_block(key, env);
    }

    // Parse a function definition
    if let Some(lambda) = obj.get("Lambda") {
        return parse_anonymous_function(lambda);
    }

    // Apply a function
    if let Some(arr) = obj.get("Application") {
        return function_application(arr, env);
    }

    // Handle conditional clauses
    if let Some(arr) = obj.get("Cond") {
        if let Value::Array(arr) = arr {
            // Expect "Clause"
            // Returns the result of the first expression where it's condition was true
            for statement in arr {
                if let Value::Array(clause) = statement.get("Clause").expect("Expect \"Clause\"") {
                    // Splits the condition and expression away
                    let [condition, expr] = clause.as_slice() else {
                        return Err(InterpError::ParseError {
                            message: "Clause did not contain both a condition and expression."
                                .to_string(),
                        });
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

        return Ok(Expr::Boolean(false));
    }

    if let Some(arr) = obj.get("Let") {
        return parse_let(arr, env);
    }

    if let Some(arr) = obj.get("Def") {
        return parse_let(arr, env);
    }

    Err(InterpError::ParseError {
        message: format!(
            "Found JSON Object in AST but it does not contain a known keyword: {:?}",
            obj
        ),
    })
}

/// Parses a block expression, handling creating a new local environment on the environment's stack
fn parse_block(val: &serde_json::Value, env: &mut Environment) -> Result<Expr, InterpError> {
    env.create_local_env();

    let res: Result<Expr, InterpError> = if let Expr::List(list) = Expr::eval(&val, env)? {
        // Return false for empty block
        Ok(list.last().cloned().unwrap_or(Expr::Boolean(false)))
    } else {
        Err(InterpError::ParseError {
            message: "Expected expressions within block.".to_string(),
        })
    };

    env.pop_top_env();
    res
}

/// Parses a block expression, handling creating a new local environment on the environment's stack
/// Special function to evaluate the block with some initial bindings (such as a function's block with arguments)
pub fn parse_block_with_bindings(
    val: &serde_json::Value,
    env: &mut Environment,
    bindings: Vec<(&String, &Expr)>,
) -> Result<Expr, InterpError> {
    env.create_local_env();
    env.bind(bindings);

    let res: Result<Expr, InterpError> = if let Expr::List(list) = Expr::eval(&val, env)? {
        // Return false for empty block
        Ok(list.last().cloned().unwrap_or(Expr::Boolean(false)))
    } else {
        Err(InterpError::ParseError {
            message: "Expected expressions within block.".to_string(),
        })
    };

    env.pop_top_env();
    res
}

fn parse_let(val: &serde_json::Value, env: &mut Environment) -> Result<Expr, InterpError> {
    let [identifier, value] = match val.as_array() {
        Some(arr) if arr.len() == 2 => [&arr[0], &arr[1]],
        _ => {
            return Err(InterpError::ParseError {
                message: "Let expression expectes an identifier and a value to be bound."
                    .to_string(),
            })
        }
    };

    let ident_name = identifier
        .get("Identifier")
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            return InterpError::ParseError {
                message: "Expecting an identifier in let expression".to_string(),
            };
        })?;

    let ident_val = Expr::eval(value, env)?;

    // Place into our environment
    env.bind(vec![(&ident_name.to_string(), &ident_val)]);

    return Ok(ident_val);
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
        .is_err_and(|e| matches!(
            e,
            InterpError::TypeError {
                expected: _,
                found: _
            }
        )));
        assert!(Expr::eval(
            &serde_json::from_str(&small_num.to_string()).unwrap(),
            &mut env
        )
        .is_err_and(|e| matches!(
            e,
            InterpError::TypeError {
                expected: _,
                found: _
            }
        )));

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

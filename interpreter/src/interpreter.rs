use std::{cell::RefCell, fmt, rc::Rc};

use serde_json::{Map, Value};

use crate::{
    environment::{Environment, LocalEnvironment},
    error::InterpError,
    functions::{function_application, parse_anonymous_function, Function},
};

/// Holds current and global environments
pub struct Interpreter {
    pub global: Environment,
    pub local: Rc<RefCell<LocalEnvironment>>,
}

impl Interpreter {
    pub fn new(lexical_scope: bool, store_output: bool) -> Self {
        let global = Environment {
            lexical_scope,
            store_output,
            output: Vec::new(),
        };
        let local = Rc::new(RefCell::new(LocalEnvironment::default_environment()));
        Self { global, local }
    }

    /// Enters a new, blank, local environment
    /// Returns the current local environment
    pub fn enter_new_local(&mut self) -> Rc<RefCell<LocalEnvironment>> {
        let old_local = self.local.clone();
        let new_local = LocalEnvironment::from_parent(old_local.clone());
        self.local = new_local;
        old_local
    }

    /// Enter specific local environment (i.e. a function's saved environment)
    /// Returns the current local environment
    pub fn enter_local(
        &mut self,
        local: Rc<RefCell<LocalEnvironment>>,
    ) -> Rc<RefCell<LocalEnvironment>> {
        let old_local = self.local.clone();
        self.local = local;
        old_local
    }
}

/// All the types of the language
#[derive(PartialEq, Eq, Clone)]
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
    pub fn eval(
        val: &serde_json::Value,
        interpreter: &mut Interpreter,
    ) -> Result<Expr, InterpError> {
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
                    .map(|val| Expr::eval(val, interpreter))
                    .collect::<Result<Vec<Expr>, InterpError>>()?,
            )),
            Value::Object(obj) => interpret_object(obj, interpreter),
            _ => Err(InterpError::ParseError {
                message: format!(
                    "{} is not an implemented type! It is of JSON type {:?}",
                    val, val
                ),
            }),
        }
    }
}

/// Interpret a JSON object, looking for the keys that correspond to certain behaviors
fn interpret_object(
    obj: &Map<String, Value>,
    interpreter: &mut Interpreter,
) -> Result<Expr, InterpError> {
    // First see if there is an identifier
    if let Some(binding) = obj.get("Identifier").and_then(|val| val.as_str()) {
        return interpreter.local.borrow().lookup(binding).ok_or_else(|| {
            InterpError::UndefinedError {
                symbol: binding.to_string(),
            }
        });
    }

    // Handle blocks
    if let Some(key) = obj.get("Block") {
        return interpret_block(key, interpreter);
    } else if let Some(lambda) = obj.get("Lambda") {
        return parse_anonymous_function(lambda, None, interpreter);
    } else if let Some(arr) = obj.get("Application") {
        return function_application(arr, interpreter);
    } else if let Some(arr) = obj.get("Cond") {
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
                    let condition = Expr::eval(condition, interpreter)?;
                    // If it is a boolean that is true, we evaluate the expression
                    if let Expr::Boolean(b) = condition {
                        if b {
                            return Expr::eval(expr, interpreter);
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
    } else if let Some(arr) = obj.get("Let") {
        let (name, var) = interpret_var(arr, interpreter, "let")?;
        // Place into new local environment
        interpreter.enter_new_local();
        interpreter.local.borrow_mut().bind(vec![(&name, &var)]);
        return Ok(var);
    } else if let Some(arr) = obj.get("Def") {
        let (name, var) = interpret_var(arr, interpreter, "def")?;
        // Place into the current local environment
        interpreter.local.borrow_mut().bind(vec![(&name, &var)]);
        return Ok(var);
    } else if let Some(arr) = obj.get("Assignment") {
        let (name, var) = interpret_var(arr, interpreter, "assignment")?;
        // Try to assign
        return interpreter.local.borrow_mut().assignment(&name, &var);
    }

    Err(InterpError::ParseError {
        message: format!(
            "Found JSON Object in AST but it does not contain a known keyword: {:?}",
            obj
        ),
    })
}

/// Interpret a block expression, handling creating a new local environment on the environment's stack
fn interpret_block(
    val: &serde_json::Value,
    interpreter: &mut Interpreter,
) -> Result<Expr, InterpError> {
    let old_local = interpreter.enter_new_local();

    let res: Result<Expr, InterpError> = if let Expr::List(list) = Expr::eval(&val, interpreter)? {
        // Return false for empty block
        Ok(list.last().cloned().unwrap_or(Expr::Boolean(false)))
    } else {
        Err(InterpError::ParseError {
            message: "Expected expressions within block.".to_string(),
        })
    };

    // Pop top local environment as we exit block
    interpreter.local = old_local;
    res
}

/// Interpret a block expression, handling creating a new local environment on the environment's stack
/// Special function to evaluate the block with some initial bindings (such as a function's block with arguments)
pub fn interpret_block_with_bindings(
    val: &serde_json::Value,
    interpreter: &mut Interpreter,
    bindings: Vec<(&String, &Expr)>,
) -> Result<Expr, InterpError> {
    let old_local = interpreter.enter_new_local();
    interpreter.local.borrow_mut().bind(bindings);

    let res: Result<Expr, InterpError> = if let Expr::List(list) = Expr::eval(&val, interpreter)? {
        // Return false for empty block
        Ok(list.last().cloned().unwrap_or(Expr::Boolean(false)))
    } else {
        Err(InterpError::ParseError {
            message: "Expected expressions within block.".to_string(),
        })
    };

    interpreter.local = old_local;
    res
}

/// Interprets creating a new variable
fn interpret_var(
    val: &serde_json::Value,
    interpreter: &mut Interpreter,
    expression_type: &str,
) -> Result<(String, Expr), InterpError> {
    let [identifier, value] =
        match val.as_array() {
            Some(arr) if arr.len() == 2 => [&arr[0], &arr[1]],
            _ => return Err(InterpError::ParseError {
                message: format!(
                    "{expression_type} expression expectes an identifier and a value to be bound."
                )
                .to_string(),
            }),
        };

    let ident_name = identifier
        .get("Identifier")
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            return InterpError::ParseError {
                message: format!("Expecting an identifier in {expression_type} expression"),
            };
        })?;

    let ident_val = Expr::eval(value, interpreter)?;

    return Ok((ident_name.to_string(), ident_val));
}

impl TryInto<bool> for Expr {
    type Error = InterpError;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let Expr::Boolean(b) = self {
            Ok(b)
        } else {
            Err(InterpError::TypeError {
                expected: "bool".to_string(),
                found: self.to_string(),
            })
        }
    }
}

impl TryInto<i64> for Expr {
    type Error = InterpError;

    fn try_into(self) -> Result<i64, Self::Error> {
        if let Expr::Integer(i) = self {
            Ok(i)
        } else {
            Err(InterpError::TypeError {
                expected: "integer".to_string(),
                found: self.to_string(),
            })
        }
    }
}

impl TryInto<String> for Expr {
    type Error = InterpError;

    fn try_into(self) -> Result<String, Self::Error> {
        if let Expr::String(s) = self {
            Ok(s.clone())
        } else {
            Err(InterpError::TypeError {
                expected: "string".to_string(),
                found: self.to_string(),
            })
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Integer(val) => write!(fmt, "{}", val),
            Expr::Boolean(val) => write!(fmt, "{}", val),
            Expr::String(val) => write!(fmt, "{}", val),
            Expr::List(list) => {
                let values: Vec<_> = list.iter().map(|v| v.to_string()).collect();
                write!(fmt, "[{}]", values.join(", "))
            }
            Expr::Function(func) => match func {
                Function::CoreFunction { name, func: _ } => write!(fmt, "function: {}", name),
                Function::Function {
                    name,
                    args: _,
                    func: _,
                    env: _,
                } => write!(fmt, "function: {}", name),
            },
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Integer(value) => write!(f, "Integer({})", value),
            Expr::Boolean(value) => write!(f, "Boolean({})", value),
            Expr::String(value) => write!(f, "String({})", value),
            Expr::List(values) => {
                let formatted_values: Vec<String> =
                    values.iter().map(|v| format!("{:?}", v)).collect();
                write!(
                    f,
                    "List([{}]), Length={}",
                    formatted_values.join(", "),
                    values.len()
                )
            }
            Expr::Function(func) => write!(f, "Function({:?})", func),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_integer() -> Result<(), InterpError> {
        let mut env = Interpreter::new(true, false);
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
        let mut env = Interpreter::new(true, false);
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
        let mut env = Interpreter::new(true, false);
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

use std::{cell::RefCell, fmt, rc::Rc};

use serde_json::Value;

use crate::{
    environment::{Environment, LocalEnvironment},
    error::InterpError,
    interpreter::{interpret_block_with_bindings, Expr, Interpreter},
};

#[derive(PartialEq, Eq, Clone)]
pub enum Function {
    // Internal Rust function (holds a function pointer)
    CoreFunction {
        name: String,
        func: fn(&[Expr], &mut Environment) -> Result<Expr, InterpError>,
    },
    // User function defined in the language. It has a name and evaluates to an expression.
    Function {
        name: String,
        args: Vec<String>,
        func: Value,
        // Copy of the environment from when this function was created (lexical scope)
        env: Rc<RefCell<LocalEnvironment>>,
    },
}

/// Parse a function into a UFunc, without evaluating it
pub fn parse_anonymous_function(
    val: &serde_json::Value,
    name: Option<&str>,
    interpreter: &mut Interpreter,
) -> Result<Expr, InterpError> {
    // We are in the "Lambda" object's value, which should have two list items, a parameters object and block object
    let [parameters, block] = match val.as_array() {
        Some(arr) if arr.len() == 2 => [&arr[0], &arr[1]],
        _ => {
            return Err(InterpError::ParseError {
                message: "Function should have an associated parameters list and block."
                    .to_string(),
            })
        }
    };

    let parameters: Vec<String> = match parameters
        .as_object()
        .and_then(|obj| obj.get("Parameters"))
        .and_then(|list| list.as_array())
    {
        Some(list) => list
            .iter()
            .map(|p| {
                match p
                    .as_object()
                    .and_then(|obj| obj.get("Identifier"))
                    .and_then(|i| i.as_str())
                {
                    Some(s) => Ok(s.to_string()),
                    _ => {
                        return Err(InterpError::ParseError {
                            message: "All parameters must be an identifier.".to_string(),
                        })
                    }
                }
            })
            .collect::<Result<Vec<String>, InterpError>>()?,
        _ => {
            return Err(InterpError::ParseError {
                message: "Parameters list is missing.".to_string(),
            })
        }
    };

    let block = block
        .as_object()
        .and_then(|obj| obj.get("Block"))
        .ok_or_else(|| {
            return InterpError::ParseError {
                message: "Function should contain a block.".to_string(),
            };
        })?;

    let local_name = if name.is_some() {
        name.unwrap()
    } else {
        "Anonymous"
    };

    let new_env = LocalEnvironment::from_parent(interpreter.local.clone());

    let expr = Function::Function {
        name: local_name.to_string(),
        args: parameters,
        func: block.clone(),
        env: new_env,
    };

    Ok(Expr::Function(expr))
}

pub fn function_application(
    val: &serde_json::Value,
    interpreter: &mut Interpreter,
) -> Result<Expr, InterpError> {
    if let Expr::List(list) = Expr::eval(val, interpreter)? {
        let (first, rest) = list.split_first().ok_or(InterpError::ParseError {
            message: "Function application on nothing.".to_string(),
        })?;
        if let Expr::Function(func) = first {
            match func {
                Function::CoreFunction { name: _name, func } => {
                    return func(rest, &mut interpreter.global)
                }
                Function::Function {
                    name,
                    args,
                    func,
                    env: local_env,
                } => {
                    if args.len() != rest.len() {
                        return Err(InterpError::ArgumentError {
                            func: name.to_string(),
                            expected: args.len(),
                            got: rest.len(),
                        });
                    }

                    // On lexical scope (default), functions use environment of where the function was originating from.
                    if interpreter.global.lexical_scope {
                        let current_local = interpreter.enter_local(local_env.clone());
                        let result = interpret_block_with_bindings(
                            &func,
                            interpreter,
                            args.into_iter()
                                .zip(rest.into_iter())
                                .collect::<Vec<(&String, &Expr)>>(),
                        );
                        // Pop environment
                        interpreter.local = current_local;
                        return result;
                    } else {
                        return interpret_block_with_bindings(
                            &func,
                            interpreter,
                            args.into_iter()
                                .zip(rest.into_iter())
                                .collect::<Vec<(&String, &Expr)>>(),
                        );
                    }
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
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::CoreFunction { name, .. } => write!(f, "CoreFunction(name: {})", name),
            Function::Function {
                name,
                args,
                func,
                env: _,
            } => {
                write!(
                    f,
                    "Function(name: {}, args: {:?}, func: {:?})",
                    name, args, func
                )
            }
        }
    }
}

/// Helper functions
fn exprs_into_i64(args: &[Expr]) -> Result<Vec<i64>, InterpError> {
    args.into_iter()
        .map(|expr| expr.clone().try_into())
        .collect::<Result<Vec<i64>, InterpError>>()
}

/// BEGIN INBUILT FUNCTIONS

// Takes in any amount of arguments and adds them together
pub fn add(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    Ok(Expr::Integer(ints.into_iter().sum()))
}

// Takes in any amount of arguments and subtracts from the first argument
pub fn sub(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    Ok(Expr::Integer(
        ints.into_iter().reduce(|first, x| first - x).unwrap_or(0),
    ))
}

// Takes in any amount of arguments and multiplies by the first argument
pub fn mul(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    Ok(Expr::Integer(ints.into_iter().product()))
}

// divides first argument by second
pub fn div(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    if ints.len() != 2 {
        return Err(InterpError::ArgumentError {
            func: "div".to_string(),
            expected: 2,
            got: ints.len(),
        });
    }
    Ok(Expr::Integer(ints[0] / ints[1]))
}

// gets remainder of first argument by second
pub fn rem(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    if ints.len() != 2 {
        return Err(InterpError::ArgumentError {
            func: "rem".to_string(),
            expected: 2,
            got: ints.len(),
        });
    }
    Ok(Expr::Integer(ints[0] % ints[1]))
}

pub fn zero(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let int = exprs_into_i64(args)?;
    let bool = int[0] == 0;
    Ok(Expr::Boolean(bool))
}

pub fn eq(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    match args.is_empty() {
        false => {
            let first = args.first().expect("Was not empty in previous check");
            Ok(Expr::Boolean(args.iter().all(|a| a.eq(first))))
        }
        true => Ok(Expr::Boolean(false)),
    }
}

pub fn greater(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    if ints.len() != 2 {
        return Err(InterpError::ArgumentError {
            func: "greater?".to_string(),
            expected: 2,
            got: ints.len(),
        });
    }
    Ok(Expr::Boolean(ints[0] > ints[1]))
}

pub fn less(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    if ints.len() != 2 {
        return Err(InterpError::ArgumentError {
            func: "less?".to_string(),
            expected: 2,
            got: ints.len(),
        });
    }
    Ok(Expr::Boolean(ints[0] < ints[1]))
}

pub fn print(args: &[Expr], global: &mut Environment) -> Result<Expr, InterpError> {
    for arg in args {
        if global.store_output {
            global.add_output(&arg.to_string());
        } else {
            print!("{}", arg);
        }
    }

    Ok(Expr::Boolean(true))
}

pub fn println(args: &[Expr], global: &mut Environment) -> Result<Expr, InterpError> {
    for arg in args {
        if global.store_output {
            let str = &mut arg.to_string();
            str.push_str("\n");
            global.add_output(str);
        } else {
            print!("{}", arg);
        }
    }
    println!();

    Ok(Expr::Boolean(true))
}

pub fn dbg(args: &[Expr], global: &mut Environment) -> Result<Expr, InterpError> {
    for arg in args {
        if global.store_output {
            global.add_output(format!("{:#?}\n", arg).as_str());
        } else {
            dbg!(arg);
        }
    }
    Ok(Expr::Boolean(true))
}

/// Returns argument strings as new, uppercase strings
/// If there are multiple arguments, it returns a list of the new strings
pub fn to_uppercase(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    if args.len() > 1 {
        let exprs = args
            .into_iter()
            .map(|f| {
                f.clone()
                    .try_into()
                    .and_then(|s: String| Ok(s.to_uppercase()))
            })
            .collect::<Result<Vec<String>, InterpError>>()?;
        Ok(Expr::List(
            exprs.into_iter().map(|s| Expr::String(s)).collect(),
        ))
    } else {
        Ok(Expr::String(
            args[0]
                .clone()
                .try_into()
                .and_then(|s: String| Ok(s.to_uppercase()))?,
        ))
    }
}

/// Returns argument strings as new, lowercase strings
/// If there are multiple arguments, it returns a list of the new strings
pub fn to_lowercase(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    if args.len() > 1 {
        let exprs = args
            .into_iter()
            .map(|f| {
                f.clone()
                    .try_into()
                    .and_then(|s: String| Ok(s.to_lowercase()))
            })
            .collect::<Result<Vec<String>, InterpError>>()?;
        Ok(Expr::List(
            exprs.into_iter().map(|s| Expr::String(s)).collect(),
        ))
    } else {
        Ok(Expr::String(
            args[0]
                .clone()
                .try_into()
                .and_then(|s: String| Ok(s.to_lowercase()))?,
        ))
    }
}

/// Concatenates strings together
pub fn concat(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let exprs = args
        .into_iter()
        .map(|f| f.clone().try_into())
        .collect::<Result<Vec<String>, InterpError>>()?;
    Ok(Expr::String(exprs.concat()))
}

/// Checks if a string contains a character
/// First argument is the character to check if the rest of the arguments contain
pub fn contains(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    let exprs = args
        .into_iter()
        .map(|f| f.clone().try_into())
        .collect::<Result<Vec<String>, InterpError>>()?;

    let first_arg = exprs.first();
    let rest = &exprs[1..];
    if first_arg.is_none() || rest.len() == 0 {
        return Err(InterpError::ArgumentError {
            func: "contains".to_string(),
            expected: 2,
            got: 0,
        });
    } else {
        let first_arg = first_arg.unwrap();
        return Ok(Expr::Boolean(
            !rest.iter().any(|ele| !ele.contains(first_arg)),
        ));
    }
}

// Returns all the arguments as a list expression
pub fn as_list(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    Ok(Expr::List(args.to_vec()))
}

// Returns the expression at the provided index of a list
// First arg: list expr
// Second arg: idx
pub fn get(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    if args.len() != 2 {
        return Err(InterpError::ArgumentError {
            func: "get".to_string(),
            expected: 2,
            got: args.len(),
        });
    }
    let idx: &i64 = &args[1].clone().try_into()?;
    if let Expr::List(list) = &args[0] {
        Ok(list[*idx as usize].clone())
    } else {
        Err(InterpError::TypeError {
            expected: "list".to_string(),
            found: args[0].to_string(),
        })
    }
}

// Sets an element in a list
// First arg: list expr
// Second arg: idx
// Thid arg: new element
pub fn set(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    if args.len() != 2 {
        return Err(InterpError::ArgumentError {
            func: "set".to_string(),
            expected: 3,
            got: args.len(),
        });
    }

    let idx: &i64 = &args[1].clone().try_into()?;
    if let Expr::List(list) = &args[0] {
        let mut new_list = list.clone();
        new_list[*idx as usize] = args[2].clone();
        Ok(Expr::List(new_list))
    } else {
        Err(InterpError::TypeError {
            expected: "list".to_string(),
            found: args[0].to_string(),
        })
    }
}

/// Gives the length of a string or a list
pub fn length(args: &[Expr], _global: &mut Environment) -> Result<Expr, InterpError> {
    match &args[0] {
        Expr::String(str) => Ok(Expr::Integer(str.len() as i64)),
        Expr::List(list) => Ok(Expr::Integer(list.len() as i64)),
        _ => Err(InterpError::TypeError {
            expected: "string or list".to_string(),
            found: args[0].to_string(),
        }),
    }
}

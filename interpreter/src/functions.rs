use std::fmt;

use serde_json::Value;

use crate::{
    environment::Environment,
    error::InterpError,
    interpreter::{interpret_block_with_bindings, Expr},
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
        env: Environment,
    },
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

/// Parse a function into a UFunc, without evaluating it
pub fn parse_anonymous_function(
    val: &serde_json::Value,
    env: &mut Environment,
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

    Ok(Expr::Function(Function::Function {
        name: "Anonymous".to_string(),
        args: parameters,
        func: block.clone(),
        env: env.clone(),
    }))
}

pub fn function_application(
    val: &serde_json::Value,
    env: &mut Environment,
) -> Result<Expr, InterpError> {
    if let Expr::List(list) = Expr::eval(val, env)? {
        let (first, rest) = list.split_first().ok_or(InterpError::ParseError {
            message: "Function application on nothing.".to_string(),
        })?;
        if let Expr::Function(func) = first {
            match func {
                Function::CoreFunction { name: _name, func } => return func(rest, env),
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
                    if env.lexical_scope {
                        return interpret_block_with_bindings(
                            &func,
                            &mut local_env.clone(),
                            args.into_iter()
                                .zip(rest.into_iter())
                                .collect::<Vec<(&String, &Expr)>>(),
                        );
                    } else {
                        return interpret_block_with_bindings(
                            &func,
                            env,
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

/// Helper functions
fn exprs_into_i64(args: &[Expr]) -> Result<Vec<i64>, InterpError> {
    args.into_iter()
        .map(|expr| expr.try_into())
        .collect::<Result<Vec<i64>, InterpError>>()
}

/// BEGIN INBUILT FUNCTIONS

// Takes in any amount of arguments and adds them together
pub fn add(args: &[Expr], _env: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    Ok(Expr::Integer(ints.into_iter().sum()))
}

// Takes in any amount of arguments and subtracts from the first argument
pub fn sub(args: &[Expr], _env: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    Ok(Expr::Integer(
        ints.into_iter().reduce(|first, x| first - x).unwrap_or(0),
    ))
}

// Takes in any amount of arguments and multiplys to the first argument
pub fn mul(args: &[Expr], _env: &mut Environment) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    Ok(Expr::Integer(ints.into_iter().product()))
}

pub fn zero(args: &[Expr], _env: &mut Environment) -> Result<Expr, InterpError> {
    let int = exprs_into_i64(args)?;
    let bool = int[0] == 0;
    Ok(Expr::Boolean(bool))
}

pub fn eq(args: &[Expr], _env: &mut Environment) -> Result<Expr, InterpError> {
    match args.is_empty() {
        false => {
            let first = args.first().expect("Was not empty in previous check");
            Ok(Expr::Boolean(args.iter().all(|a| a.eq(first))))
        }
        true => Ok(Expr::Boolean(false)),
    }
}

pub fn print(args: &[Expr], env: &mut Environment) -> Result<Expr, InterpError> {
    for arg in args {
        if env.store_output {
            env.add_output(&arg.to_string());
        } else {
            print!("{}", arg);
        }
    }

    Ok(Expr::Boolean(true))
}

pub fn println(args: &[Expr], env: &mut Environment) -> Result<Expr, InterpError> {
    for arg in args {
        if env.store_output {
            let str = &mut arg.to_string();
            str.push_str("\n");
            env.add_output(str);
        } else {
            println!("{}", arg);
        }
    }

    Ok(Expr::Boolean(true))
}

pub fn dbg(args: &[Expr], env: &mut Environment) -> Result<Expr, InterpError> {
    for arg in args {
        if env.store_output {
            env.add_output(format!("{:#?}\n", arg).as_str());
        } else {
            dbg!(arg);
        }
    }
    Ok(Expr::Boolean(true))
}

/// Returns argument strings as new, uppercase strings
/// If there are multiple arguments, it returns a list of the new strings
pub fn to_uppercase(args: &[Expr], _env: &mut Environment) -> Result<Expr, InterpError> {
    if args.len() > 1 {
        let exprs = args
            .into_iter()
            .map(|f| f.try_into().and_then(|s: String| Ok(s.to_uppercase())))
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
pub fn to_lowercase(args: &[Expr], _env: &mut Environment) -> Result<Expr, InterpError> {
    if args.len() > 1 {
        let exprs = args
            .into_iter()
            .map(|f| f.try_into().and_then(|s: String| Ok(s.to_lowercase())))
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
pub fn concat(args: &[Expr], _env: &mut Environment) -> Result<Expr, InterpError> {
    let exprs = args
        .into_iter()
        .map(|f| f.try_into())
        .collect::<Result<Vec<String>, InterpError>>()?;
    Ok(Expr::String(exprs.concat()))
}

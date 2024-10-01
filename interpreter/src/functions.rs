use serde_json::Value;

use crate::{
    environment::Environment,
    error::InterpError,
    interpreter::{exprs_into_i64, parse_block_with_bindings, Expr},
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Function {
    // Internal Rust function (holds a function pointer)
    RFunc {
        name: String,
        func: fn(&[Expr]) -> Result<Expr, InterpError>,
    },
    // User function defined in the language. It has a name and evaluates to an expression.
    UFunc {
        name: String,
        args: Vec<String>,
        func: Value,
    },
}

/// Parse a function into a UFunc, without evaluating it
pub fn parse_anonymous_function(val: &serde_json::Value) -> Result<Expr, InterpError> {
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

    Ok(Expr::Function(Function::UFunc {
        name: "Anonymous".to_string(),
        args: parameters,
        func: block.clone(),
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
                Function::RFunc { name: _name, func } => return func(rest),
                Function::UFunc {
                    name: _name,
                    args,
                    func,
                } => {
                    return parse_block_with_bindings(
                        func,
                        env,
                        args.into_iter()
                            .zip(rest.into_iter())
                            .collect::<Vec<(&String, &Expr)>>(),
                    )
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

/// BEGIN INBUILT FUNCTIONS

// Takes in any amount of arguments and adds them together
pub fn add(args: &[Expr]) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    Ok(Expr::Integer(ints.into_iter().sum()))
}

// Takes in any amount of arguments and subtracts from the first argument
pub fn sub(args: &[Expr]) -> Result<Expr, InterpError> {
    let ints = exprs_into_i64(args)?;
    Ok(Expr::Integer(
        ints.into_iter().reduce(|first, x| first - x).unwrap_or(0),
    ))
}

pub fn zero(args: &[Expr]) -> Result<Expr, InterpError> {
    let int = exprs_into_i64(args)?;
    let bool = int[0] == 0;
    Ok(Expr::Boolean(bool))
}

pub fn eq(args: &[Expr]) -> Result<Expr, InterpError> {
    match args.is_empty() {
        false => {
            let first = args.first().expect("Was not empty in previous check");
            Ok(Expr::Boolean(args.iter().all(|a| a.eq(first))))
        }
        true => Ok(Expr::Boolean(false)),
    }
}

pub fn print(args: &[Expr]) -> Result<Expr, InterpError> {
    for arg in args {
        print!("{}", arg);
    }

    Ok(Expr::Boolean(true))
}

pub fn println(args: &[Expr]) -> Result<Expr, InterpError> {
    for arg in args {
        println!("{}", arg);
    }

    Ok(Expr::Boolean(true))
}

pub fn dbg(args: &[Expr]) -> Result<Expr, InterpError> {
    for arg in args {
        dbg!(arg);
    }
    Ok(Expr::Boolean(true))
}

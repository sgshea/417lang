use serde_json::Value;

use crate::{error::InterpError, interpreter::{exprs_into_i64, Expr}};

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
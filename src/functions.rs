use crate::interpreter::{exprs_into_i64, Expr, InterpError};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Function {
    // Will eventually have another variant for functions created in the language
    // Internal Rust function (holds a function pointer)
    RFunc(fn(&[Expr]) -> Result<Expr, InterpError>),
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
        ints.into_iter().reduce(|first, x| {
            first - x
        }).unwrap_or(0)
    ))
}
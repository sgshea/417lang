use environment::Environment;
use error::InterpError;
use interpreter::Expr;

use wasm_bindgen::prelude::*;

pub mod environment;
pub mod error;
mod functions;
pub mod interpreter;

/// Interprets a string, assumed to be valid JSON input from a parser
/// Returns either the interpreted expression or an error
pub fn interpret_string(input: &str) -> Result<Expr, InterpError> {
    // Interpret input
    match serde_json::from_str(input) {
        Err(_) => Err(InterpError::ParseError {
            message: "Unable to parse JSON into interpreter.".to_string(),
        }),
        Ok(j) => interpret_default(j),
    }
}

pub fn interpret_default(val: serde_json::Value) -> Result<Expr, InterpError> {
    let env = &mut Environment::default_environment();
    Expr::eval(&val, env)
}

/// Parses and then interprets a string
/// Returns the result of parsing and interpreting in string form
#[cfg(feature = "parser")]
#[wasm_bindgen]
pub fn interpret_with_parser_to_string(input: &str) -> String {
    use parser::parse;
    match parse("input", input) {
        Err(e) => return format!("{:?}", e.as_diagnostic()),
        Ok(ast) => match interpret_default(ast) {
            Err(e) => return e.to_string(),
            Ok(expr) => return expr.to_string(),
        },
    }
}

/// Macro to use the interpreter & parser together
#[cfg(feature = "parser")]
#[allow(unused_macros)]
macro_rules! language {
    ($($input:tt)*) => {{
        let mut env = Environment::default_environment();
        let code_as_string = stringify!($($input)*);
        interpret(parse("test", code_as_string), &mut env)
    }};
}

/// Parsing for the entire interpreter
/// Uses parser feature to simulate input
#[cfg(feature = "parser")]
#[cfg(test)]
mod tests {
    use error::InterpError;
    use interpreter::Expr;

    use parser::parse;

    use super::*;

    #[test]
    fn checkpoint_2() -> Result<(), InterpError> {
        assert_eq!(language! { -2 }?, Expr::Integer(-2));
        assert_eq!(language! { 000 }?, Expr::Integer(0));
        assert_eq!(language! { add(4, 5) }?, Expr::Integer(9));
        assert_eq!(
            language! { add(1, add(2, add(3, add(4, 5)))) }?,
            Expr::Integer(15)
        );

        Ok(())
    }

    #[test]
    fn checkpoint_3() -> Result<(), InterpError> {
        assert_eq!(language! { sub(0, 1) }?, Expr::Integer(-1));
        assert_eq!(language! { cond (true => 5) }?, Expr::Integer(5));
        assert_eq!(
            language! { cond (false => -1) (true => 5) }?,
            Expr::Integer(5)
        );
        assert!(language! { cond (add(1, 1) => -1) (true => 5) }.is_err());

        Ok(())
    }
}

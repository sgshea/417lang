mod environment;
mod error;
mod functions;
mod interpreter;

use std::{error::Error, io};

use environment::Environment;
use interpreter::interpret;

pub fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "parser")]
    {
        use parser::parse;

        let input = io::read_to_string(io::stdin());
        let ast = parse(&input.expect("Error reading input."));
        let mut env = Environment::default_environment();
        match interpret(ast, &mut env) {
            Err(e) => Err(Box::new(e)),
            Ok(expr) => {
                println!("{}", expr);
                Ok(())
            }
        }
    }

    #[cfg(not(feature = "parser"))]
    {
        use error::InterpError;
        use serde_json::Value;
        let input = match serde_json::from_reader(io::stdin()) {
            Err(e) => {
                // We don't want to error on empty input
                if e.is_eof() {
                    Ok(Value::String("".to_string()))
                } else {
                    Err(InterpError::ParseError {
                        message: e.to_string(),
                    })
                }
            }
            Ok(ok) => Ok(ok),
        };
        // Initialize the environment
        let mut env = Environment::default_environment();

        // Interpret input
        match input {
            Err(e) => Err(Box::new(e)),
            Ok(val) => match interpret(val, &mut env) {
                Err(e) => Err(Box::new(e)),
                Ok(expr) => {
                    println!("{}", expr);
                    Ok(())
                }
            },
        }
    }
}

/// Macro to use the interpreter & parser together
#[cfg(feature = "parser")]
macro_rules! language {
    ($($input:tt)*) => {{
        let mut env = Environment::default_environment();
        let code_as_string = stringify!($($input)*);
        interpret(parse(code_as_string), &mut env)
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

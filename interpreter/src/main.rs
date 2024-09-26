mod interpreter;
mod environment;
mod error;
mod functions;

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
            Ok(_) => Ok(()),
        }
    }

    #[cfg(not(feature = "parser"))]
    {
        use serde_json::Value;
        use error::InterpError;
        let input = match serde_json::from_reader(io::stdin()) {
            Err(e) => {
                // We don't want to error on empty input
                if e.is_eof() {
                    Ok(Value::String("".to_string()))
                } else {
                    Err(InterpError::ParseError { message: e.to_string() })
                }
            }
            Ok(ok) => Ok(ok),
        };
        // Initialize the environment
        let mut env = Environment::default_environment();

        // Interpret input
        match input {
            Err(e) => Err(Box::new(e)),
            Ok(val) => {
                match interpret(val, &mut env) {
                    Err(e) => Err(Box::new(e)),
                    Ok(_) => Ok(()),
                }
            },
        }
    }
}


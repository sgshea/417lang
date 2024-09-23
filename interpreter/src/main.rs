mod reader;
mod interpreter;
mod environment;
mod functions;

use std::error::Error;

use environment::Environment;
use interpreter::interpret;

pub fn main() -> Result<(), Box<dyn Error>> {

    #[cfg(feature = "parser")]
    {
        use std::{env, io};
        use parser::parse;
        let args: Vec<String> = env::args().collect();
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
        let input = reader::read_stdin();
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


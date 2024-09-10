mod reader;
mod interpreter;
mod environment;
mod functions;

use std::error::Error;

use environment::Environment;
use interpreter::interpret;

fn main() -> Result<(), Box<dyn Error>> {
    // Read JSON from input
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


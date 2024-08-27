use std::io;

use crate::types::{LError, LType};

/// Read only a single value
pub fn rep() -> Result<(), LError> {
    let mut input_buffer = String::new();
    let res = get_line(&mut input_buffer);
    match res {
        Ok(value) => {
            println!("{}", value);
        },
        Err(e) => {
            return Err(e)
        }
    }
    input_buffer.clear();

    Ok(())
}

/// REPL behavior for command line usage
pub fn repl() -> Result<(), LError> {
    let mut input_buffer = String::new();
    loop {
        let res = get_line(&mut input_buffer);
        match res {
            Ok(value) => {
                println!("{}", value);
            },
            Err(e) => {
                return Err(e)
            }
        }
        input_buffer.clear();
    }
}

/// Get a single line from stdin
/// Try to interpret into LType
pub fn get_line(buffer: &mut String) -> Result<LType, LError> {
    match io::stdin().read_line(buffer) {
        Ok(_) => LType::interpret_as_value(&buffer),
        Err(e) => Err(LError::IOError(e)),
    }
}

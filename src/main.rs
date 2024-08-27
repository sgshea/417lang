mod repl;
mod types;

use std::env;
use repl::{rep, repl};
use types::LError;

fn main() -> Result<(), LError> {
    // If argument "--repl" is passed in, evaluate in a loop
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--repl".to_string()) {
        repl()
    } else {
        rep()
    }
}


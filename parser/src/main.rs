use std::io;

use miette::Result;
use parser::parse;

/// Simple executable main function for the parser.
/// Takes in input from stdin and pretty-prints it
fn main() -> Result<()> {
    let input = io::read_to_string(io::stdin());
    let ast = parse(&input.expect("Error reading input."))?;

    println!("{}", serde_json::to_string_pretty(&ast).unwrap());
    Ok(())
}
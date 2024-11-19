use lexer::{Lexer, Token};
use parser::Parser;
use serde_json::Value;

pub mod error;
mod lexer;
mod parser;

pub fn parse(source_name: &str, input: &str) -> Result<Value, error::ParseError> {
    let mut lexer = Lexer::new(source_name, input);

    let mut tokens = vec![];
    loop {
        let token = lexer.next_token_container();
        if token.token == Token::EOF {
            tokens.push(token);
            break;
        }
        tokens.push(token);
    }

    Parser::new(source_name, input, &tokens).parse_program()
}
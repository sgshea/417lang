use lexer::{Lexer, Token};
use parser::Parser;
use serde_json::Value;

pub mod error;
mod lexer;
mod parser;

pub fn parse(input: &str) -> Result<Value, error::ParseError> {
    let mut lexer = Lexer::new(input);

    let mut tokens = vec![];
    loop {
        let token = lexer.next_token_container();
        if token.token == Token::EOF {
            tokens.push(token);
            break;
        }
        tokens.push(token);
    }

    Parser::new(input, &tokens).parse_program()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn factorial() {
        let input = r#"
        // Factorial
        def fact λ(n)
        {
        cond 
            (zero?(n) => 1) 
            (true => mul(n, fact(sub(n, 1))))
        }
        // And there can be comments at the end
        "#;

        let output = r#"
        {"Def":[{"Identifier": "fact"},{"Lambda":[{"Parameters":[{"Identifier": "n"}]},{"Block":[{"Cond":[{"Clause":[{"Application":[{"Identifier": "zero?"},{"Identifier": "n"}]},1]},{"Clause":[{"Identifier": "true"},{"Application":[{"Identifier": "mul"},{"Identifier": "n"},{"Application":[{"Identifier": "fact"},{"Application":[{"Identifier": "sub"},{"Identifier": "n"},1]}]}]}]}]}]}]}]}
        "#;
        let output_json: Value = serde_json::from_str(output).expect("should be valid json");

        let ast = parse(input).unwrap();
        println!("{}", serde_json::to_string_pretty(&ast).unwrap());
        println!("{}", serde_json::to_string_pretty(&output_json).unwrap());
        assert_eq!(ast, output_json);
    }

    #[test]
    fn defs() {
        let input = r#"
        {def foo 1; let bar 2; def baz 3}
        "#;
        let output = r#"
        {"Block":[{"Def":[{"Identifier": "foo"},1]},{"Let":[{"Identifier": "bar"},2]},{"Def":[{"Identifier": "baz"},3]}]}
        "#;

        let output_json: Value = serde_json::from_str(output).expect("should be valid json");

        let ast = parse(input).unwrap();
        println!("{}", serde_json::to_string_pretty(&ast).unwrap());
        println!("{}", serde_json::to_string_pretty(&output_json).unwrap());
        assert_eq!(ast, output_json);
    }
}

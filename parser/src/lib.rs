mod lexer;
mod parser;

#[cfg(test)]
mod tests {
    use lexer::{Lexer, Token};
    use parser::Parser;
    use serde_json::Value;

    use super::*;

    #[test]
    fn factorial() {
        let input = r#"
        // Example Factorial program from original parser
        def fact λ(n) {
            cond 
                (zero?(n) => 1) 
                (true => mul(n, fact(sub(n, 1))))
        }
        "#;

        let output = r#"
        {"Def":[{"Identifier": "fact"},
        {"Lambda":[{"Parameters":[{"Identifier": "n"}]},
        {"Block":[{"Cond":[{"Clause":[{"Application":[{"Identifier": "zero?"},
        {"Identifier": "n"}]},1]},{"Clause":[{"Identifier": "true"},
        {"Application":[{"Identifier": "mul"},{"Identifier": "n"},
        {"Application":[{"Identifier": "fact"},
        {"Application":[{"Identifier": "sub"},
        {"Identifier": "n"},1]}]}]}]}]}]}]}]}
        "#;
        let output_json: Value = serde_json::from_str(output).expect("should be valid json");
        let mut lexer = Lexer::new(input);

        let mut tokens = vec![];
        loop {
            let token = lexer.next_token();
            if token == Token::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse_program();
        println!("{}", serde_json::to_string_pretty(&ast).unwrap());
        println!("{}", serde_json::to_string_pretty(&output_json).unwrap());
        assert_eq!(ast, output_json);
    }
}

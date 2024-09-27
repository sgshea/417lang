use serde_json::{json, Value};

use crate::lexer::{Keyword, Token};

// Define a simple parser structure
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, current: 0 }
    }

    // Utility function to get the current token
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }

    // Utility function to move to the next token
    fn next_token(&mut self) {
        self.current += 1;
    }

    // Match and consume a token
    fn consume(&mut self, token: &Token) -> bool {
        if self.current_token() == token {
            self.next_token();
            true
        } else {
            false
        }
    }

    // Entry point for parsing a program (EXP := FORM | ATOM)
    pub fn parse_program(&mut self) -> Value {
        self.parse_exp()
    }

    // EXP := FORM | ATOM
    fn parse_exp(&mut self) -> Value {
        let mut expr = match self.current_token() {
            Token::Identifier(_) | Token::Integer(_) | Token::String(_) => self.parse_atom(),
            Token::Keyword(_) => self.parse_form(),
            Token::OpenBrace => self.parse_block(),
            _ => panic!("Unexpected token: {:?}", self.current_token()),
        };
    
        // If the next token is an OpenParen, treat it as a function application
        if let Token::OpenParen = self.current_token() {
            expr = self.parse_application(expr);
        }
    
        expr
    }

    // FORM := APPLICATION | LAMBDA | COND | BLOCK | LET | DEFINITION
    fn parse_form(&mut self) -> Value {
        match self.current_token() {
            Token::Keyword(kw) => {
                match kw {
                    Keyword::Def => self.parse_definition(),
                    Keyword::Let => self.parse_let(),
                    Keyword::Lambda => self.parse_lambda(),
                    Keyword::Cond => self.parse_cond(),
                }
            }
            _ => panic!("Unexpected form: {:?}", self.current_token()),
        }
    }

    // ATOM := IDENTIFIER | STRING | INTEGER
    fn parse_atom(&mut self) -> Value {
        match self.current_token() {
            Token::Identifier(ref name) => {
                let value = json!({ "Identifier": name });
                self.next_token(); // Consume the identifier
                value
            }
            Token::Integer(ref num) => {
                let value = json!(num);
                self.next_token(); // Consume the integer
                value
            }
            Token::String(ref s) => {
                let value = json!(s);
                self.next_token(); // Consume the string
                value
            }
            _ => panic!("Unexpected atom: {:?}", self.current_token()),
        }
    }

    // APPLICATION := EXP '(' ARGLIST? ')'
    fn parse_application(&mut self, func: Value) -> Value {
        self.consume(&Token::OpenParen); // Expect '('
        // It is a flat vector with the function identifier as the first element
        let mut args = vec![func];

        // If there are arguments, parse them
        if !self.consume(&Token::CloseParen) {
            loop {
                args.push(self.parse_exp()); // Parse each argument
                if self.consume(&Token::CloseParen) {
                    break;
                }
                self.consume(&Token::Comma); // Optional ',' between arguments
            }
        }

        // Construct the JSON for the function application
        json!({ "Application": args })
    }

    // LAMBDA := ('lambda' | 'λ') '(' PARAMETERS ')' BLOCK
    fn parse_lambda(&mut self) -> Value {
        self.next_token(); // Consume 'lambda' or 'λ'
        self.consume(&Token::OpenParen); // Expect '('
        let params = self.parse_parameters(); // Parse parameters
        self.consume(&Token::CloseParen); // Expect ')'
        let block = self.parse_block(); // Parse block
        json!({ "Lambda": [params, block] })
    }

    // PARAMETERS := IDENTIFIER (',' IDENTIFIER)*
    fn parse_parameters(&mut self) -> Value {
        let mut params = vec![];
        while let Token::Identifier(ref name) = self.current_token() {
            params.push(json!({ "Identifier": name }));
            self.next_token(); // Consume the identifier
            if !self.consume(&Token::Comma) {
                break;
            }
        }
        json!({ "Parameters": params })
    }

    // COND := 'cond' CLAUSE+
    fn parse_cond(&mut self) -> Value {
        self.next_token(); // Consume 'cond'
        let mut clauses = vec![];
        while let Token::OpenParen = self.current_token() {
            clauses.push(self.parse_clause());
        }
        json!({ "Cond": clauses })
    }

    // CLAUSE := '(' EXP '=>' EXP ')'
    fn parse_clause(&mut self) -> Value {
        self.consume(&Token::OpenParen); // Expect '('
        let condition = self.parse_exp(); // Parse the condition
        self.consume(&Token::Arrow); // Expect '=>'
        let result = self.parse_exp(); // Parse the result
        self.consume(&Token::CloseParen); // Expect ')'
        json!({ "Clause": [condition, result] })
    }

    // BLOCK := '{' EXPLIST? '}'
    fn parse_block(&mut self) -> Value {
        self.consume(&Token::OpenBrace); // Expect '{'
        let mut exps = vec![];
        while !self.consume(&Token::CloseBrace) { // Expect '}' to end
            exps.push(self.parse_exp());
            if self.consume(&Token::Semicolon) {
                continue;
            }
        }
        json!({ "Block": exps })
    }

    // LET := 'let' IDENTIFIER EXP
    fn parse_let(&mut self) -> Value {
        self.consume(&Token::Keyword(Keyword::Let)); // Expect 'let'
        let identifier = self.parse_atom();
        let exp = self.parse_exp();
        json!({ "Let": [identifier, exp] })
    }

    // DEFINITION := 'def' IDENTIFIER EXP
    fn parse_definition(&mut self) -> Value {
        self.consume(&Token::Keyword(Keyword::Def));
        let name = self.parse_atom();
        let body = self.parse_exp();
        json!({ "Def": [name, body] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parser() {
        let tokens = vec![
            Token::Keyword(Keyword::Def),
            Token::Identifier("fact".to_string()),
            Token::Keyword(Keyword::Lambda),
            Token::OpenParen,
            Token::Identifier("n".to_string()),
            Token::CloseParen,
            Token::OpenBrace,
            Token::Keyword(Keyword::Cond),
            Token::OpenParen,
            Token::Identifier("zero?".to_string()),
            Token::OpenParen,
            Token::Identifier("n".to_string()),
            Token::CloseParen,
            Token::Arrow,
            Token::Integer(1),
            Token::CloseParen,
            Token::OpenParen,
            Token::Identifier("true".to_string()),
            Token::Arrow,
            Token::Identifier("mul".to_string()),
            Token::OpenParen,
            Token::Identifier("n".to_string()),
            Token::Comma,
            Token::Identifier("fact".to_string()),
            Token::OpenParen,
            Token::Identifier("sub".to_string()),
            Token::OpenParen,
            Token::Identifier("n".to_string()),
            Token::Comma,
            Token::Integer(1),
            Token::CloseParen,
            Token::CloseParen,
            Token::CloseParen,
            Token::CloseParen,
            Token::CloseBrace,
            Token::EOF,
        ];

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse_program();

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
        assert_eq!(ast, output_json);
    }
}
use miette::LabeledSpan;
use serde_json::{json, Value};

use crate::{
    error::{ParseError, ParseErrorType},
    lexer::{Keyword, LexToken, Token},
};

// Define a simple parser structure
pub struct Parser<'a, T: LexToken> {
    tokens: &'a [T],
    current: usize,

    source_name: &'a str,
    source: &'a str,
}

impl<'a, T: LexToken> Parser<'a, T> {
    pub fn new(source_name: &'a str, source: &'a str, tokens: &'a [T]) -> Self {
        Parser {
            tokens,
            current: 0,
            source_name,
            source,
        }
    }

    // Utility function to get the current token
    fn current_token(&self) -> &Token {
        self.tokens[self.current].token()
    }

    fn current_source(&self) -> Option<usize> {
        self.tokens[self.current].source()
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
    pub fn parse_program(&mut self) -> Result<Value, ParseError> {
        self.parse_exp()
    }

    // EXP := FORM | ATOM
    fn parse_exp(&mut self) -> Result<Value, ParseError> {
        let expr = match self.current_token() {
            Token::Identifier(_) | Token::Integer(_) | Token::String(_) => self.parse_atom(),
            Token::Keyword(_) => self.parse_form(),
            Token::OpenBrace => self.parse_block(),
            _ => {
                let err = ParseError::new(
                    crate::error::ParseErrorType::BLOCK,
                    &self.source_name,
                    &self.source,
                    (self.current_source().unwrap(), 1).into(),
                    "Expected expression",
                );
                return Err(err);
            }
        };

        // If the next token is an OpenParen, treat it as a function application
        if let Token::OpenParen = self.current_token() {
            return self.parse_application(expr?);
        }

        expr
    }

    // FORM := APPLICATION | LAMBDA | COND | BLOCK | LET | DEFINITION
    fn parse_form(&mut self) -> Result<Value, ParseError> {
        match &self.current_token() {
            Token::Keyword(kw) => match kw {
                Keyword::Def => self.parse_definition(),
                Keyword::Let => self.parse_let(),
                Keyword::Lambda => self.parse_lambda(),
                Keyword::Cond => self.parse_cond(),
            },
            _ => panic!("Unexpected form: {:?}", self.current_token()),
        }
    }

    // ATOM := IDENTIFIER | STRING | INTEGER
    fn parse_atom(&mut self) -> Result<Value, ParseError> {
        match self.current_token().clone() {
            Token::Identifier(_) => self.parse_assignment(),
            Token::Integer(ref num) => {
                let value = json!(num);
                self.next_token(); // Consume the integer
                Ok(value)
            }
            Token::String(ref s) => {
                let value = json!(s);
                self.next_token(); // Consume the string
                Ok(value)
            }
            _ => panic!("Unexpected atom: {:?}", self.current_token()),
        }
    }

    // APPLICATION := EXP '(' ARGLIST? ')'
    fn parse_application(&mut self, func: Value) -> Result<Value, ParseError> {
        self.consume(&Token::OpenParen); // Expect '('
                                         // It is a flat vector with the function identifier as the first element
        let mut args = vec![func];

        // If there are arguments, parse them
        if !self.consume(&Token::CloseParen) {
            loop {
                args.push(self.parse_exp()?);
                if self.consume(&Token::CloseParen) {
                    break;
                }
                self.consume(&Token::Comma); // Optional ',' between arguments
            }
        }

        // Construct the JSON for the function application
        Ok(json!({ "Application": args }))
    }

    // LAMBDA := ('lambda' | 'λ') '(' PARAMETERS ')' BLOCK
    fn parse_lambda(&mut self) -> Result<Value, ParseError> {
        self.next_token(); // Consume 'lambda' or 'λ'
        self.consume(&Token::OpenParen); // Expect '('
        let params = self.parse_parameters(); // Parse parameters
        self.consume(&Token::CloseParen); // Expect ')'
        let block = self.parse_block()?; // Parse block
        Ok(json!({ "Lambda": [params, block] }))
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
    fn parse_cond(&mut self) -> Result<Value, ParseError> {
        self.next_token(); // Consume 'cond'
        let mut clauses = vec![];
        while let Token::OpenParen = self.current_token() {
            clauses.push(self.parse_clause()?);
        }
        Ok(json!({ "Cond": clauses }))
    }

    // CLAUSE := '(' EXP '=>' EXP ')'
    fn parse_clause(&mut self) -> Result<Value, ParseError> {
        self.consume(&Token::OpenParen); // Expect '('
        let condition = self.parse_exp()?; // Parse the condition
        self.consume(&Token::Arrow); // Expect '=>'
        let result = self.parse_exp()?; // Parse the result
        self.consume(&Token::CloseParen); // Expect ')'
        Ok(json!({ "Clause": [condition, result] }))
    }

    // BLOCK := '{' EXPLIST? '}'
    fn parse_block(&mut self) -> Result<Value, ParseError> {
        let current_source = self.current_source(); // used to construct error if needed
        if !self.consume(&Token::OpenBrace) {
            // Expect '{'
            return Err(ParseError::new_full(
                crate::error::ParseErrorType::BLOCK,
                &self.source_name,
                &self.source,
                (current_source.unwrap(), 1).into(),
                "Expected a block",
                Some("Create a block with enclosing braces".to_string()),
                vec![],
            ));
        }
        let mut exps = vec![];
        while !self.consume(&Token::CloseBrace) {
            // Expect '}' to end
            match self.parse_exp() {
                Ok(exp) => exps.push(exp),
                Err(mut e) => {
                    match e.error_type {
                        ParseErrorType::BLOCK => {
                            // customize error message
                            e.change_label("Found end of block");
                            let start_block_span = LabeledSpan::at(
                                current_source.expect("Expect block start source to exist"),
                                "Found opening '{' here",
                            );
                            e.add_spans(&mut vec![start_block_span]);
                            e.add_help("Close the block with a '}'");
                        }
                        _ => {}
                    }
                    return Err(e);
                }
            }
            if self.consume(&Token::Semicolon) {
                continue;
            }
        }
        Ok(json!({ "Block": exps }))
    }

    // LET := 'let' IDENTIFIER '=' EXP
    fn parse_let(&mut self) -> Result<Value, ParseError> {
        self.consume(&Token::Keyword(Keyword::Let)); // Expect 'let'
        let identifier = self.parse_identifier()?;
        if !self.consume(&Token::Equals) {
            // Expect '='
            return Err(ParseError::new_full(
                crate::error::ParseErrorType::LET,
                &self.source_name,
                &self.source,
                (self.current_source().unwrap() + 1, 1).into(),
                "Expected an '='",
                Some("Let expression has form 'let x = 5'".to_string()),
                vec![],
            ));
        }
        let exp = self.parse_exp()?;
        Ok(json!({ "Let": [identifier, exp] }))
    }

    // DEFINITION := 'def' IDENTIFIER EXP
    fn parse_definition(&mut self) -> Result<Value, ParseError> {
        self.consume(&Token::Keyword(Keyword::Def));
        let name = self.parse_atom()?;
        let body = self.parse_exp()?;
        Ok(json!({ "Def": [name, body] }))
    }

    /// Helper function to parse an identifier when it is expected
    fn parse_identifier(&mut self) -> Result<Value, ParseError> {
        match self.current_token().clone() {
            Token::Identifier(ref name) => {
                let ident = json!({ "Identifier": name });
                self.next_token(); // Consume the identifier
                Ok(ident)
            }
            _ => {
                return Err(ParseError::new_full(
                    crate::error::ParseErrorType::UNEXPECTED,
                    &self.source_name,
                    &self.source,
                    (self.current_source().unwrap(), 2).into(),
                    "Expected an identifier",
                    Some("A valid identifier starts with a valid unicode character, but not a digit, '+' or '-'.".to_string()),
                    vec![],
                ));
            }
        }
    }

    // ASSIGNMENT := IDENTIFIER '=' EXP
    // This gets called from parse_atom, we already have the identifier name
    // This function tests if there is an equals after the identifier
    fn parse_assignment(&mut self) -> Result<Value, ParseError> {
        // Get identifier
        let ident = self.parse_identifier()?;
        // Check to see if next token is an equals token
        if self.consume(&Token::Equals) {
            // Return assignment
            let body = self.parse_exp()?;
            Ok(json!({ "Assignment": [ident, body]}))
        } else {
            // Else just return the identifier
            Ok(ident)
        }
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

        // passing in empty input (we don't expect errors for this test anyway)
        let mut parser = Parser::new("", "", &tokens);
        let ast = parser.parse_program().unwrap();

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

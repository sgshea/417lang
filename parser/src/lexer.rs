use std::{collections::HashMap, str::Chars};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    String(String),
    Keyword(Keyword),
    Integer(i64),
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Comma,
    Semicolon,
    Arrow,
    EOF,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Lambda,
    Let,
    Def,
    Cond,
}

pub struct Lexer<'a> {
    _source: &'a str,
    input: Chars<'a>,
    keywords: HashMap<&'a str, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("lambda", Token::Keyword(Keyword::Lambda));
        keywords.insert("λ", Token::Keyword(Keyword::Lambda));
        keywords.insert("let", Token::Keyword(Keyword::Let));
        keywords.insert("def", Token::Keyword(Keyword::Def));
        keywords.insert("cond", Token::Keyword(Keyword::Cond));

        let lexer = Self {
            _source: source,
            input: source.chars(),
            keywords,
        };
        lexer
    }

     // Move to the next character in the input
     fn next_char(&mut self) {
        self.input.next();
    }

    // Peek the next character without consuming it
    fn peek_char(&mut self) -> Option<char> {
        self.input.clone().next()
    }

    // Skip all whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else if c == '/' && self.peek_char() == Some('/') {
                // Skip comments until the end of the line
                while self.peek_char() != Some('\n') {
                    self.next_char();
                }
            } else {
                break;
            }
        }
    }

    // Lex the next token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        match self.peek_char() {
            Some('"') => self.lex_string(),
            Some('(') => {
                self.next_char();
                Token::OpenParen
            }
            Some(')') => {
                self.next_char();
                Token::CloseParen
            }
            Some('{') => {
                self.next_char();
                Token::OpenBrace
            }
            Some('}') => {
                self.next_char();
                Token::CloseBrace
            }
            Some(',') => {
                self.next_char();
                Token::Comma
            }
            Some(';') => {
                self.next_char();
                Token::Semicolon
            }
            Some('=') => {
                // Make sure we check for '=>' before we try to lex for identifier or keyword as '=' is accepted for that
                let mut forward = self.input.clone();
                forward.next();
                if forward.next() == Some('>') {
                    // Consume both '=' and '>'
                    self.input = forward;
                    Token::Arrow
                } else {
                    // We can assume it is an identifer or keyword now
                    self.lex_identifier_or_keyword()
                }
            }
            Some(c) if is_id_start(c) => self.lex_identifier_or_keyword(),
            Some(c) if c.is_digit(10) || c == '+' || c == '-' => self.lex_integer(),
            Some(_) => panic!("Unexpected character: {:?}", self.peek_char()),
            None => Token::EOF,
        }
    }

    // Lex an identifier or a keyword
    fn lex_identifier_or_keyword(&mut self) -> Token {
        let mut identifier = String::new();

        // First character must be valid IDSTART
        if let Some(c) = self.peek_char() {
            if is_id_start(c) {
                identifier.push(c);
                self.next_char();
            } else {
                panic!("Invalid identifier start character: {:?}", c);
            }
        }

        // Continue consuming IDCHARs
        while let Some(c) = self.peek_char() {
            if is_id_char(c) {
                identifier.push(c);
                self.next_char();
            } else {
                break;
            }
        }

        match self.keywords.get(identifier.as_str()) {
            None => Token::Identifier(identifier),
            Some(keyword) => keyword.clone(),
        }
    }

    // Lex an integer (positive or negative)
    fn lex_integer(&mut self) -> Token {
        let mut num_str = String::new();

        if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
            num_str.push(self.peek_char().unwrap());
            self.next_char();
        }

        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                num_str.push(c);
                self.next_char();
            } else {
                break;
            }
        }

        let num: i64 = num_str.parse().unwrap();
        Token::Integer(num)
    }

    // Lex a string (handles escape sequences)
    fn lex_string(&mut self) -> Token {
        let mut string_content = String::new();
        self.next_char(); // Consume the opening quote

        while let Some(c) = self.peek_char() {
            match c {
                '"' => {
                    self.next_char(); // Consume the closing quote
                    break;
                }
                '\\' => {
                    self.next_char(); // Consume the escape character
                    if let Some(escaped_char) = self.peek_char() {
                        match escaped_char {
                            '\\' => string_content.push('\\'),
                            '"' => string_content.push('"'),
                            't' => string_content.push('\t'),
                            'n' => string_content.push('\n'),
                            'r' => string_content.push('\r'),
                            _ => panic!("Invalid escape sequence: \\{}", escaped_char),
                        }
                        self.next_char();
                    }
                }
                _ => {
                    string_content.push(c);
                    self.next_char();
                }
            }
        }

        Token::String(string_content)
    }
}

// Helper functions

// Check if a character is a valid IDSTART
fn is_id_start(c: char) -> bool {
    // IDSTART must be a valid Unicode character, but not a digit, '+' or '-'
    is_id_char(c) && !(c.is_digit(10) || match c {
        '+' | '-' => true,
        _ => false
    })
}

// Check if a character is a valid IDCHAR
fn is_id_char(c: char) -> bool {
    // IDCHAR is any valid UTF-8 character except for delimiters
    !(c.is_whitespace() || is_delimiter(c))
}

// Check if a character is a delimiter
fn is_delimiter(c: char) -> bool {
    match c {
        ' ' | '\t' | '\n' | '"' | '(' | ')' | '{' | '}' | ',' | ';' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer() {
        let input = r#"
        // Example Factorial program from original parser
        def fact λ(n) {
            cond 
                (zero?(n) => 1) 
                (true => mul(n, fact(sub(n, 1))))
        }
        "#;
        // let input = r#"add"#;
        let mut lexer = Lexer::new(input);

        loop {
            let token = lexer.next_token();
            println!("{:?}", token);
            if token == Token::EOF {
                break;
            }
        }
    }
}
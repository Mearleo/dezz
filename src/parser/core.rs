use crate::lexer::{SpannedToken, Token};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.token)
    }

    pub fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).map(|t| t.token.clone());
        self.pos += 1;
        tok
    }

    pub fn next_span(&mut self) -> Option<&SpannedToken> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    pub fn expect(&mut self, expected: &Token) {
        match self.next_span() {
            Some(span) if span.token == *expected => {
                // OK: token matches, nothing to do
            }
            Some(span) => {
                panic!(
                    "Line {}, Col {}: Expected {:?}, got {:?}",
                    span.line, span.column, expected, span.token
                );
            }
            None => {
                panic!("Unexpected end of input: expected {:?}", expected);
            }
        }
    }

    pub fn token_err(&self, msg: &str) -> ! {
        if let Some(token) = self.tokens.get(self.pos) {
            panic!(
                "Line {}, Col {}: {}",
                token.line, token.column, msg
            );
        } else {
            panic!("Unexpected end of input: {}", msg);
        }
    }
}

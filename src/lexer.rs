#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(f64),
    Equal,
    Smaller,
    Greater,
    Tilde,
    Action,
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    Setting,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Semicolon,
    String(String),
    Range,
}

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    tokens: Vec<SpannedToken>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
            tokens: Vec::new(),
            line: 1,
            column: 1,
        }
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        Some(c)
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn push(&mut self, token: Token) {
        self.tokens.push(SpannedToken { token, line: self.line, column: self.column });
    }
}

impl<'a> Lexer<'a> {
    pub fn tokenize(mut self) -> Vec<SpannedToken> {
        while let Some(&c) = self.peek() {
            let start_line = self.line;
            let start_col = self.column;

            match c {
                ' ' | '\t' | '\n' => { self.next(); }
                '=' => {
                    self.push(Token::Equal);
                    self.next();
                }
                '<' => {
                    self.push(Token::Smaller);
                    self.next();
                }
                '>' => {
                    self.push(Token::Greater);
                    self.next();
                }
                '~' => {
                    self.push(Token::Tilde);
                    self.next();
                }
                '+' => {
                    self.push(Token::Plus);
                    self.next();
                }
                '-' => {
                    self.next();
                    if let Some(&next) = self.peek() {
                        if next == '>' {
                            self.push(Token::Action);
                            self.next();
                        } else {
                            self.push(Token::Minus);
                        }
                    }
                }
                '*' => {
                    self.push(Token::Multiply);
                    self.next();
                }
                '/' => {
                    self.next(); // consume first '/'

                    if let Some(next) = self.peek() {
                        match next {
                            '/' => {
                                // Line comment: //
                                self.next(); // consume second '/'
                                while let Some(&c) = self.peek() {
                                    if c == '\n' {
                                        break;
                                    }
                                    self.next();
                                }
                            }
                            '*' => {
                                // Block comment: /* ... */
                                self.next(); // consume '*'

                                loop {
                                    match self.next() {
                                        Some('*') => {
                                            // possible end of comment
                                            if let Some('/') = self.peek() {
                                                self.next(); // consume '/'
                                                break;
                                            }
                                        }
                                        Some(_) => {
                                            // keep consuming
                                        }
                                        None => {
                                            panic!("Unterminated block comment");
                                        }
                                    }
                                }
                            }
                            _ => {
                                // It's just a divide operator
                                self.push(Token::Divide);
                            }
                        }
                    } else {
                        // '/' at EOF → treat as divide
                        self.push(Token::Divide);
                    }
                }
                '^' => {
                    self.push(Token::Power);
                    self.next();
                }
                '@' => {
                    self.push(Token::Setting);
                    self.next();
                }
                '(' => {
                    self.push(Token::LeftParen);
                    self.next();
                }
                ')' => {
                    self.push(Token::RightParen);
                    self.next();
                }
                '{' => {
                    self.push(Token::LeftBrace);
                    self.next();
                }
                '}' => {
                    self.push(Token::RightBrace);
                    self.next();
                }
                '[' => {
                    self.push(Token::LeftBracket);
                    self.next();
                }
                ']' => {
                    self.push(Token::RightBracket);
                    self.next();
                }
                ':' => {
                    self.push(Token::Colon);
                    self.next();
                }
                ';' => {
                    self.push(Token::Semicolon);
                    self.next();
                }
                ',' => {
                    self.push(Token::Comma);
                    self.next();
                }
                '"' => {
                    self.next();

                    let mut string = String::new();

                    while let Some(&d) = self.peek() {
                        if d != '"' {
                            string.push(d);
                            self.next();
                        } else {
                            break;
                        }
                    }

                    self.next();

                    self.push(Token::String(string));
                }
                '0'..='9' => {
                    let mut num = String::new();
                    while let Some(&d) = self.peek() {
                        if d.is_ascii_digit() {
                            self.next();
                            num.push(d);
                        } else if d == '.' {
                            self.next();
                            if matches!(self.peek(), Some('.')) {
                                self.next();
                                if matches!(self.next(), Some('.')) {
                                    self.push(Token::Number(num.parse().unwrap()));
                                    num.clear();
                                    self.push(Token::Range);
                                } else {
                                    panic!("Expected a third point for the Range operator.");
                                }
                            } else {
                                num.push(d);
                            }
                        } else {
                            break;
                        }
                    }
                    self.push(Token::Number(num.parse().unwrap()));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::new();
                    while let Some(&d) = self.peek() {
                        if d.is_alphanumeric() || d == '_' {
                            ident.push(d);
                            self.next();
                        } else {
                            break;
                        }
                    }
                    self.push(Token::Ident(ident));
                }
                '.' => {
                    if !matches!(self.next(), Some('.')) {
                        if !matches!(self.next(), Some('.')) {
                            self.push(Token::Range);
                        }
                    }
                }
                _ => panic!("Unexpected character '{}' at {}:{}", c, start_line, start_col),
            }
        }

        self.tokens
    }
}

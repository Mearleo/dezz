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

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' => { chars.next(); }
            '=' => {
                tokens.push(Token::Equal);
                chars.next();
            }
            '<' => {
                tokens.push(Token::Smaller);
                chars.next();
            }
            '>' => {
                tokens.push(Token::Greater);
                chars.next();
            }
            '~' => {
                tokens.push(Token::Tilde);
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                chars.next();
                if let Some(&next) = chars.peek() {
                    if next == '>' {
                        tokens.push(Token::Action);
                        chars.next();
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
            }
            '*' => {
                tokens.push(Token::Multiply);
                chars.next();
            }
            '/' => {
                chars.next(); // consume first '/'

                if let Some(&next) = chars.peek() {
                    match next {
                        '/' => {
                            // Line comment: //
                            chars.next(); // consume second '/'
                            while let Some(&c) = chars.peek() {
                                if c == '\n' {
                                    break;
                                }
                                chars.next();
                            }
                        }
                        '*' => {
                            // Block comment: /* ... */
                            chars.next(); // consume '*'

                            loop {
                                match chars.next() {
                                    Some('*') => {
                                        // possible end of comment
                                        if let Some(&'/') = chars.peek() {
                                            chars.next(); // consume '/'
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
                            tokens.push(Token::Divide);
                        }
                    }
                } else {
                    // '/' at EOF → treat as divide
                    tokens.push(Token::Divide);
                }
            }
            '^' => {
                tokens.push(Token::Power);
                chars.next();
            }
            '@' => {
                tokens.push(Token::Setting);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen);
                chars.next();
            }
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '"' => {
                chars.next();

                let mut string = String::new();

                while let Some(&d) = chars.peek() {
                    if d != '"' {
                        string.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }

                chars.next();

                tokens.push(Token::String(string));
            }

            '0'..='9' => {
                let mut num = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        chars.next();
                        num.push(d);
                    } else if d == '.' {
                        chars.next();
                        if matches!(chars.peek(), Some('.')) {
                            chars.next();
                            if matches!(chars.next(), Some('.')) {
                                tokens.push(Token::Number(num.parse().unwrap()));
                                num.clear();
                                tokens.push(Token::Range);
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
                tokens.push(Token::Number(num.parse().unwrap()));
            }
            'a'..='z' | 'A'..='Z' => {
                let mut ident = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_alphanumeric() || d == '_' {
                        ident.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(
                    ident
                ));
            }
            '.' => {
                if !matches!(chars.next(), Some('.')) {
                    if !matches!(chars.next(), Some('.')) {
                        tokens.push(Token::Range);
                    }
                }
            }
            _ => panic!("Unexpected character: {}", c),
        }
    }

    tokens
}

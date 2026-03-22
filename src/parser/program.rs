use crate::lexer::Token;
use crate::ast::*;
use super::core::*;

// parse program
impl Parser {
    pub fn parse_program(&mut self) -> Graph {
        let mut graph = Graph::new();

        // parse global settings
        while let Some(token) = self.peek() {
            match token {
                Token::Setting => { self.parse_global_setting(&mut graph); }
                _ => break
            }
        }

        // parse items
        while let Some(_) = self.peek() {
            graph.items.push(self.parse_item());
        }

        graph
    }
}

// parse global settings
impl Parser {
    pub fn parse_global_setting(&mut self, graph: &mut Graph) {
        self.next();
        
        if let Some(token) = self.next() {
            match token {
                Token::Ident(name) => {
                    match name.as_str() {
                        "Viewport" => { graph.viewport = Some(self.parse_viewport()) }
                        "Ticker" => { graph.ticker = Some(self.parse_ticker()) }
                        _ => panic!("There is no global setting named: {:?}", name),
                    }
                }
                _ => panic!("Expected Ident, instead got: {:?}", token),
            }
        }
    }

    pub fn parse_viewport(&mut self) -> Viewport {
        let mut viewport = Viewport::default();

        self.expect(&Token::LeftBrace);

        while let Some(token) = self.next() {
            match token {
                Token::Ident(string) => {
                    match string.as_str() {
                        "xmin" => {
                            self.expect(&Token::Colon);
                            viewport.xmin = self.parse_expression();
                        }
                        "xmax" => {
                            self.expect(&Token::Colon);
                            viewport.xmax = self.parse_expression();
                        }
                        "ymin" => {
                            self.expect(&Token::Colon);
                            viewport.ymin = self.parse_expression();
                        }
                        "ymax" => {
                            self.expect(&Token::Colon);
                            viewport.ymax = self.parse_expression();
                        }
                        _ => panic!("Unknown setting: {}", string),
                    }
                }
                Token::Comma => continue,
                Token::RightBrace => break,
                _ => panic!("Unexpected token in Setting: {:?}", token)
            }
        }

        return viewport
    }

    pub fn parse_ticker(&mut self) -> Ticker {
        let mut ticker = Ticker::default();

        self.expect(&Token::LeftBrace);

        while let Some(token) = self.next() {
            match token {
                Token::Ident(string) => {
                    match string.as_str() {
                        "run" => {
                            self.expect(&Token::Colon);
                            ticker.run = Some(self.parse_expression());
                        }
                        "step" => {
                            self.expect(&Token::Colon);
                            ticker.step = Some(self.parse_expression());
                        }
                        _ => panic!("Unknown setting: {}", string),
                    }
                }
                Token::Comma => continue,
                Token::RightBrace => break,
                _ => panic!("Unexpected token in Setting: {:?}", token)
            }
        }

        return ticker
    }
}

// parse item
impl Parser {
    pub fn parse_item(&mut self) -> Item {
        let item = match self.peek() {
            Some(Token::String(_)) => self.parse_note_or_folder(),
            _ => Item::Expression(self.parse_expression()),
        };

        if matches!(self.peek(), Some(Token::Semicolon)) {
            self.next();
        }
        
        item
    }
}

// parse note / folder
impl Parser {
    pub fn parse_note_or_folder(&mut self) -> Item {
        let name = match self.next() {
            Some(Token::String(s)) => s,
            _ => unreachable!(),
        };

        return match self.peek() {
            Some(Token::LeftBrace) => {
                self.next(); // consume {

                let mut items = Vec::new();

                while let Some(token) = self.peek() {
                    match token {
                        Token::RightBrace => break,
                        _ => { items.push(self.parse_item()); }
                    }
                }
                
                self.expect(&Token::RightBrace);
                
                Item::Folder(Folder { title: name, items })
            }
            Some(Token::Semicolon) => {
                self.next();
                Item::Note(Note { text: name })
            }
            _ => Item::Note(Note { text: name }),
        }
    }
}
use crate::lexer::Token;
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

// Parser frame
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) {
        let t = self.next();
        if t.as_ref() != Some(expected) {
            panic!("Expected {:?}, got {:?}", expected, t);
        }
    }
}

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

// global settings
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

// items
impl Parser {
    fn parse_item(&mut self) -> Item {
        let item = match self.peek() {
            Some(Token::String(_)) => self.parse_note_or_folder(),
            _ => Item::Expression(self.parse_expression()),
        };

        if matches!(self.peek(), Some(Token::Semicolon)) {
            self.next();
        }
        
        item
    }

    fn parse_note_or_folder(&mut self) -> Item {
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

    pub fn parse_expression(&mut self) -> Expression {
        self.parse_setting()
    }

    pub fn parse_setting(&mut self) -> Expression {
        let expr = self.parse_equality();

        if let Some(Token::Setting) = self.peek() {
            self.next();

            self.expect(&Token::LeftBrace);

            let mut setting = Setting::new(expr);

            while let Some(token) = self.next() {
                match token {
                    Token::Ident(string) => {
                        match string.as_str() {
                            "color" => {
                                self.expect(&Token::Colon);
                                if let Some(Token::String(s)) = self.peek() {
                                    setting.color = s.into();
                                    self.next();
                                } else if let Some(Token::Ident(s)) = self.peek() {
                                    setting.color_latex = Some(Expression::Ident(s.into()).to_string());
                                    self.next();
                                } else {
                                    panic!("Expected String or Ident, found: {:?}", self.next())
                                }
                            }
                            "min" => {
                                self.expect(&Token::Colon);
                                let expr = self.parse_expression();

                                let slider = setting.slider.get_or_insert_with(SliderSetting::new);
                                slider.min = Some(Box::new(expr));
                            }
                            "max" => {
                                self.expect(&Token::Colon);
                                let expr = self.parse_expression();

                                let slider = setting.slider.get_or_insert_with(SliderSetting::new);
                                slider.max = Some(Box::new(expr));
                            }
                            "step" => {
                                self.expect(&Token::Colon);
                                let expr = self.parse_expression();

                                let slider = setting.slider.get_or_insert_with(SliderSetting::new);
                                slider.step = Some(Box::new(expr));
                            }
                            "lineWidth" => {
                                self.expect(&Token::Colon);
                                let expr = self.parse_expression();
                                setting.line_width = Some(Box::new(expr));
                            }
                            "lineOpacity" => {
                                self.expect(&Token::Colon);
                                let expr = self.parse_expression();
                                setting.line_opacity= Some(Box::new(expr));
                            }
                            "pointSize" => {
                                self.expect(&Token::Colon);
                                let expr = self.parse_expression();
                                setting.point_size = Some(Box::new(expr));
                            }
                            "pointOpacity" => {
                                self.expect(&Token::Colon);
                                let expr = self.parse_expression();
                                setting.point_opacity = Some(Box::new(expr));
                            }
                            _ => panic!("Unknown setting: {}", string),
                        }
                    }
                    Token::Comma => continue,
                    Token::RightBrace => break,
                    _ => panic!("Unexpected token in Setting: {:?}", token)
                }
            }

            Expression::Setting(setting)
        } else {
            expr
        }
    }

    // equality: = | ~
    fn parse_equality(&mut self) -> Expression {
        let mut expr = self.parse_condition();

        loop {
            match self.peek() {
                Some(Token::Equal) => {
                    self.next();
                    let right = self.parse_condition();
                    expr = Expression::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                Some(Token::Tilde) => {
                    self.next();
                    let right = self.parse_condition();
                    expr = Expression::Binary {
                        op: BinaryOp::Regression,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    // parse condition: expr{bool}
    fn parse_condition(&mut self) -> Expression {
        let mut expr = self.parse_if();

        loop {
            match self.peek() {
                Some(Token::LeftBrace) => {
                    self.next();
                    let right = self.parse_expression();
                    expr = Expression::Binary {
                        op: BinaryOp::Condition,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                    self.expect(&Token::RightBrace);
                }
                _ => break,
            }
        }

        expr
    }

    // if: comparison:expr
    fn parse_if(&mut self) -> Expression {
        let mut expr = self.parse_comparison();

        loop {
            match self.peek() {
                Some(Token::Colon) => {
                    self.next();
                    let right = self.parse_expression();
                    expr = Expression::Binary {
                        op: BinaryOp::Then,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    // comparison: < | >
    fn parse_comparison(&mut self) -> Expression {
        let mut expr = self.parse_action();

        loop {
            match self.peek() {
                Some(Token::Smaller) => {
                    self.next();
                    let right = self.parse_expression();
                    expr = Expression::Binary {
                        op: BinaryOp::Smaller,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                Some(Token::Greater) => {
                    self.next();
                    let right = self.parse_expression();
                    expr = Expression::Binary {
                        op: BinaryOp::Greater,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    // action: term -> term
    fn parse_action(&mut self) -> Expression {
        let mut expr = self.parse_range();

        loop {
            match self.peek() {
                Some(Token::Action) => {
                    self.next();
                    let right = self.parse_range();
                    expr = Expression::Binary {
                        op: BinaryOp::Action,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    // parse range
    fn parse_range(&mut self) -> Expression {
        let mut expr = self.parse_term();

        loop {
            match self.peek() {
                Some(Token::Range) => {
                    self.next();
                    let right = self.parse_term();
                    expr = Expression::Binary {
                        op: BinaryOp::Range,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    // term: factor (("+" | "-") factor)*
    fn parse_term(&mut self) -> Expression {
        let mut expr = self.parse_factor();

        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.next();
                    let right = self.parse_factor();
                    expr = Expression::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                Some(Token::Minus) => {
                    self.next();
                    let right = self.parse_factor();
                    expr = Expression::Binary {
                        op: BinaryOp::Sub,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    // factor: power (("*" | "/") power)*
    fn parse_factor(&mut self) -> Expression {
        let mut expr = self.parse_power();

        loop {
            match self.peek() {
                Some(Token::Multiply) => {
                    self.next();
                    let right = self.parse_power();
                    expr = Expression::Binary {
                        op: BinaryOp::Mul,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                Some(Token::Divide) => {
                    self.next();
                    let right = self.parse_power();
                    expr = Expression::Binary {
                        op: BinaryOp::Div,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    // power: unary ("^" unary)*
    fn parse_power(&mut self) -> Expression {
        let mut expr = self.parse_unary();

        loop {
            match self.peek() {
                Some(Token::Power) => {
                    self.next();
                    let right = self.parse_unary();
                    expr = Expression::Binary {
                        op: BinaryOp::Pow,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    // unary: ("+" | "-") unary | primary
    fn parse_unary(&mut self) -> Expression {
        match self.peek() {
            Some(Token::Plus) => {
                self.next();
                Expression::Unary {
                    op: UnaryOp::Plus,
                    expr: Box::new(self.parse_unary()),
                }
            }
            Some(Token::Minus) => {
                self.next();
                Expression::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(self.parse_unary()),
                }
            }
            _ => self.parse_primary(),
        }
    }

    // primary: number | ident | call | "(...)" | "{...}"
    fn parse_primary(&mut self) -> Expression {
        match self.next() {
            Some(Token::Number(n)) => Expression::Number(n),

            Some(Token::Ident(name)) => { // call or ident or index
                // if function call
                if let Some(Token::LeftParen) = self.peek() {
                    self.next();
                    let mut args = Vec::new();

                    if !matches!(self.peek(), Some(Token::RightParen)) {
                        loop {
                            if matches!(self.peek(), Some(Token::RightParen)) {
                                break;
                            }
                            args.push(self.parse_expression());
                            if !matches!(self.peek(), Some(Token::Comma)) {
                                break;
                            }
                            self.next();
                        }
                    }

                    self.expect(&Token::RightParen);

                    Expression::Call { ident: name, args }
                } else if let Some(Token::LeftBracket) = self.peek() {
                    let list = self.parse_expression();

                    Expression::Binary { op: BinaryOp::Index, left: Box::new(Expression::Ident(name)), right: Box::new(list) }
                } else {
                    Expression::Ident(name)
                }
            }

            Some(Token::LeftParen) => { // group or point or action block or action collection
                if matches!(self.peek(), Some(Token::RightParen)) { //  if action block: ()->{...}
                    self.next();
                    self.expect(&Token::Action);
                    self.expect(&Token::LeftBrace);

                    let mut actions: Vec<Expression> = Vec::new();

                    while !matches!(self.peek(), Some(Token::RightBrace)) {
                        actions.push(self.parse_expression());
                    }

                    self.expect(&Token::RightBrace);

                    Expression::ActionBlock(actions)
                } else {
                    let expr = self.parse_expression();

                    // if point or action collection
                    if matches!(self.peek(), Some(Token::Comma)) {
                        self.next();

                        // if action collection
                        if matches!(expr, Expression::Binary { op: BinaryOp::Action, .. }) {
                            let mut actions: Vec<Expression> = vec![expr];

                            loop {
                                if matches!(self.peek(), Some(Token::RightParen)) {
                                    break;
                                }
                                actions.push(self.parse_expression());
                                if !matches!(self.peek(), Some(Token::Comma)) {
                                    break;
                                }
                                self.next();
                            }

                            self.expect(&Token::RightParen);

                            Expression::ActionCollection(actions)
                        } else { // else point
                            let y = self.parse_expression();
                            self.expect(&Token::RightParen);
                            Expression::Point(Point { x: Box::new(expr), y: Box::new(y) })
                        }
                    } else { // else group
                        self.expect(&Token::RightParen);
                        Expression::Group(Box::new(expr))
                    }
                }
            }

            Some(Token::LeftBrace) => { // conditional
                let mut ifs: Vec<Expression> = Vec::new();
                
                loop {
                    if matches!(self.peek(), Some(Token::RightBrace)) {
                        break;
                    }
                    ifs.push(self.parse_expression());
                    if !matches!(self.peek(), Some(Token::Comma)) {
                        break;
                    }
                    self.next();
                }

                self.expect(&Token::RightBrace);
                Expression::Conditional(ifs)
            }

            Some(Token::LeftBracket) => {
                let mut items: Vec<Expression> = Vec::new();
                
                while let Some(token) = self.peek() {
                    match token {
                        Token::RightBracket => {
                            self.next();
                            break;
                        }
                        Token::Comma => {
                            self.next();
                            continue;
                        }
                        _ => {
                            items.push(self.parse_expression());
                        }
                    }
                } 

                Expression::List(items)
            }

            other => panic!("Unexpected token in primary: {:?}", other),
        }
        
    }
}

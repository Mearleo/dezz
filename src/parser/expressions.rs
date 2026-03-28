use crate::lexer::Token;
use crate::ast::*;
use super::core::*;

// items
impl Parser {
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
        let mut expr = self.parse_suffixes();

        loop {
            match self.peek() {
                Some(Token::Equal) => {
                    self.next();
                    let right = self.parse_suffixes();
                    expr = Expression::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                Some(Token::Tilde) => {
                    self.next();
                    let right = self.parse_suffixes();
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

    // parse for or with
    fn parse_suffixes(&mut self) -> Expression {
        let mut expr = self.parse_condition();

        if let Some(Token::Ident(string)) = self.peek() {
            match string.as_str() {
                "_for" => {
                    self.next();

                    let mut args: Vec<Expression> = Vec::new();

                    loop {
                        args.push(self.parse_expression());

                        if matches!(self.peek(), Some(Token::Comma)) {
                            self.next();
                        } else {
                            break;
                        }
                    }

                    let right = Expression::BlankList(args);
                    expr = Expression::Binary {
                        op: BinaryOp::For,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                "_with" => {
                    self.next();

                    let mut args: Vec<Expression> = Vec::new();

                    loop {
                        args.push(self.parse_expression());

                        if matches!(self.peek(), Some(Token::Comma)) {
                            self.next();
                        } else {
                            break;
                        }
                    }

                    let right = Expression::BlankList(args);
                    expr = Expression::Binary {
                        op: BinaryOp::With,
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => {}
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

            Some(Token::LeftParen) => { // group (expr), point (expr, expr), action block (..args)->{...}, action collection (action, action, ...)

                let mut args: Vec<Expression> = Vec::new();

                while !matches!(self.peek(), Some(Token::RightParen)) {
                    args.push(self.parse_expression());

                    if !matches!(self.peek(), Some(Token::Comma)) {
                        break;
                    }
                    self.next();
                }

                self.expect(&Token::RightParen);
                
                if matches!(args.first(), Some(Expression::Binary { op: BinaryOp::Action, ..})) { // if action collection
                    return Expression::ActionCollection(args)
                }

                if matches!(self.peek(), Some(Token::Action)) { // if action block
                    self.next();
                    self.expect(&Token::LeftBrace);

                    let mut actions: Vec<Expression> = Vec::new();

                    while !matches!(self.peek(), Some(Token::RightBrace)) {
                        actions.push(self.parse_expression());
                    }

                    self.expect(&Token::RightBrace);

                    return Expression::ActionBlock { args, actions }
                } else { // else point, group or action collection
                    if args.len() == 1 {
                        return Expression::Group(Box::new(args[0].clone()))
                    } else if args.len() == 2 {
                        return Expression::Point(Point { x: Box::new(args[0].clone()), y: Box::new(args[1].clone()) })
                    } else {
                        return Expression::ActionCollection(args)
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

            other => self.token_err(format!("Unexpected token in primary: {:?}", other).as_str()),
        }
        
    }
}


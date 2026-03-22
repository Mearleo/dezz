use serde::Serialize;
use std::{ fmt };

use crate::ast::action_block::resolve_action_block;

// Whole Desmos Graph
#[derive(Serialize, Debug, Clone)]
pub struct Graph {
    pub viewport: Option<Viewport>,
    pub ticker: Option<Ticker>,
    pub items: Vec<Item>,
}

impl Graph {
    pub fn simplify(&mut self) {
        for item in &mut self.items {
            item.simplify();
        }
    }

    pub fn to_base(&mut self) {
        let new_items: &mut Vec<Item> = &mut Vec::new();

        for item in self.items.clone() {
            new_items.extend(item.to_base());
        }

        self.items = new_items.clone();
    }
}

impl Graph {
    pub fn new() -> Self {
        Self { viewport: None, ticker: None, items: Vec::new() }
    }
}


// Viewport Settings
#[derive(Serialize, Debug, Clone)]
pub struct Viewport {
    pub xmin: Expression,
    pub ymin: Expression,
    pub xmax: Expression,
    pub ymax: Expression,
    pub complex: bool,
}

impl Viewport {
    pub fn default() -> Self {
        Self {
            xmin: Expression::Number(-10.0),
            xmax: Expression::Number(10.0),
            ymin: Expression::Number(-10.0),
            ymax: Expression::Number(10.0),
            complex: false,
        }
    }
}

// Ticker
#[derive(Serialize, Debug, Clone)]
pub struct Ticker {
    pub run: Option<Expression>,
    pub step: Option<Expression>,
}

impl Ticker {
    pub fn default() -> Self {
        Self {
            run: None,
            step: None,
        }
    }
}

// The Cell/Item
#[derive(Serialize, Debug, Clone)]
pub enum Item {
    Expression(Expression),
    Folder(Folder),
    Note(Note),
}

impl Item {
    fn simplify(&mut self) {
        if let Item::Expression(expr) = self {
            expr.simplify();
        }
    }

    fn to_base(&self) -> Vec<Item> {
        match self {
            Item::Note(note) => {
                vec![Item::Note(note.clone())]
            }
            Item::Folder(folder) => {
                let new_items: &mut Vec<Item> = &mut Vec::new();

                for item in folder.items.clone() {
                    new_items.extend(item.to_base());
                }

                vec![
                    Item::Folder(Folder { title: folder.title.clone(), items: new_items.clone() })
                ]
            }
            Item::Expression(expression) => {
                // convert Vec<Expression> into Vec<Item>
                expression.to_base().iter().map(|expr| Item::Expression(expr.clone())).collect()
            }
        }
    }
}

// item type: Expression
#[derive(Serialize, Debug, Clone)]
pub enum Expression {
    Setting(Setting),
    Number(f64),
    Ident(String),
    Point(Point),

    // Function call: f(x), sin(x), etc.
    Call {
        ident: String,
        args: Vec<Expression>,
    },

    // Binary operators: +, -, *, /, ^, etc.
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    // Unary operators: -x, +x
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },

    // Parentheses or grouping
    Group(Box<Expression>),

    // Lists [...]
    List(Vec<Expression>),

    // Action Blocks ()->{...actions}
    ActionBlock(Vec<Expression>),

    // Action Collection (...actions)
    ActionCollection(Vec<Expression>),

    // Condition ifs,else_val (e.g {x<0:-x,x>0:x,5})
    Conditional(Vec<Expression>), // bool:expr (e.g. 2<3:1)
}

impl Expression {
    pub fn walk_mut(&mut self, f: &mut impl FnMut(&mut Expression) -> bool) {
        if !f(self) {
            return;
        }

        match self {
            Expression::Binary { left, right, .. } => {
                left.walk_mut(f);
                right.walk_mut(f);
            }
            Expression::Group(inner) => {
                inner.walk_mut(f);
            }
            Expression::ActionBlock(items) => {
                for item in items {
                    item.walk_mut(f);
                }
            }
            Expression::Setting(setting) => {
                setting.expr.walk_mut(f);
            }
            Expression::Unary { expr, .. } => {
                expr.walk_mut(f);
            }
            Expression::Call { args, .. } => {
                for arg in args {
                    arg.walk_mut(f);
                }
            }
            Expression::Ident(_) => {}
            Expression::Number(_) => {}
            other => {
                println!("{:?}", other);
                todo!()
            }
        }
    }
}


impl Expression {
    fn to_base(&self) -> Vec<Expression> {
        let mut expressions: Vec<Expression> = Vec::new();

        match self {
            Expression::ActionBlock(items) => {
                expressions.extend(
                    resolve_action_block(items.clone())
                );
            }
            Expression::Binary { op, left, right } => {
                let mut left_items = left.to_base().into_iter();
                let mut right_items = right.to_base().into_iter();

                let left_first = left_items.next().unwrap();
                let right_first = right_items.next().unwrap();

                expressions.push(Expression::Binary {
                    op: op.clone(),
                    left: Box::new(left_first),
                    right: Box::new(right_first),
                });

                expressions.extend(left_items);
                expressions.extend(right_items);
            }
            _ => {
                expressions.push(self.clone());
            }
        }

        expressions
    }

    pub fn simplify(&mut self) {
        self.walk_mut(&mut |node| {
            match node {
                Expression::Group(inner) => {
                    // Collapse nested groups
                    if let Expression::Group(grandchild) = &mut **inner {
                        *node = Expression::Group(grandchild.clone());
                    }
                }

                Expression::Binary { op, left, right } => {
                    // Remove unnecessary groups
                    if matches!(op, BinaryOp::Div | BinaryOp::Eq) {
                        if let Expression::Group(inner) = &mut **left {
                            *left = inner.clone();
                        }
                        if let Expression::Group(inner) = &mut **right {
                            *right = inner.clone();
                        }
                    } else if matches!(op, BinaryOp::Pow) {
                        if let Expression::Group(inner) = &mut **right {
                            *right = inner.clone();
                        }
                    }
                }

                _ => {}
            }

            true
        });
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Expression::Binary { op, left, right } => {
                match op {
                    BinaryOp::Div => format!("\\frac{{{}}}{{{}}}", left, right),
                    BinaryOp::Pow => format!("{}{}{{{}}}", left, op, right),
                    BinaryOp::Condition => format!("{}\\{{{}\\}}", left, right),
                    _ => format!("{}{}{}", left, op, right),
                }
            }
            Expression::Call { ident, args } => {
                let args_str = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                // convert keywords
                let ident = match ident.as_str() {
                    "sin" => "\\sin".into(),
                    "cos" => "\\cos".into(),
                    "tan" => "\\tan".into(),
                    "csc" => "\\csc".into(),
                    "sec" => "\\sec".into(),
                    "cot" => "\\cot".into(),
                    "arcsin" => "\\arcsin".into(),
                    "arccos" => "\\arccos".into(),
                    "arctan" => "\\arctan".into(),
                    "arccsc" => "\\arccsc".into(),
                    "arcsec" => "\\arcsec".into(),
                    "arccot" => "\\arccot".into(),
                    "abs" => "\\operatorname{abs}".into(),
                    "random" => "\\operatorname{random}".into(),
                    "polygon" => "\\operatorname{polygon}".into(),
                    "rgb" => "\\operatorname{rgb}".into(),
                    _ => format!("f_{{{}}}", ident),
                }.to_string();

                format!("{}({})", ident, args_str)
            }
            Expression::Group(expr) => {
                format!("({})", expr)
            }
            Expression::Conditional( ifs ) => {
                let ifs = ifs
                    .iter().map(|item| item.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("\\{{{}\\}}", ifs)
            }
            Expression::ActionCollection(actions) => {
                let actions = actions
                    .iter().map(|item| item.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}", actions)
            }
            Expression::Ident(ident) => {
                // convert keywords (important!)
                let ident = match ident.as_str() {
                    "x" => "x".into(),
                    "y" => "y".into(),
                    "pi" => "\\pi".into(),
                    "tau" => "\\tau".into(),
                    "e" => "e".into(),
                    "infty" => "\\infty".into(),
                    "infinity" => "\\infty".into(),
                    "r" => "r".into(),
                    "theta" => "\\theta".into(),
                    "t" => "t".into(),
                    "width" => "\\operatorname{width}".into(),
                    "height" => "\\operatorname{height}".into(),
                    "dt" => "\\operatorname{dt}".into(),
                    _ => format!("v_{{{}}}", ident),
                }.to_string();

                ident
            }
            Expression::Number(number) => {
                number.to_string()
            }
            Expression::Unary { op, expr } => {
                format!("{}{}", op, expr)
            }
            Expression::Point(point ) => {
                format!("({},{})", point.x, point.y)
            }
            Expression::List(items) => {
                let items_str = items
                    .iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", items_str)
            }
            _ => panic!("Unexpected display attempt for {:?}", self)
        };

        write!(f, "{}", s)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Setting {
    pub expr: Box<Expression>,
    pub color: String,
    pub color_latex: Option<String>,
    pub slider: Option<SliderSetting>,
    pub line_width: Option<Box<Expression>>,
    pub line_opacity: Option<Box<Expression>>,
    pub point_size: Option<Box<Expression>>,
    pub point_opacity: Option<Box<Expression>>,
}

impl Setting {
    pub fn new(expr: Expression) -> Self {
        Self {
            expr: Box::new(expr),
            color: "#000".into(),
            color_latex: None,
            slider: None,
            line_width: None,
            line_opacity: None,
            point_size: None,
            point_opacity: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SliderSetting {
    pub min: Option<Box<Expression>>,
    pub max: Option<Box<Expression>>,
    pub step: Option<Box<Expression>>,
}

impl SliderSetting {
    pub fn new() -> Self {
        return Self { min: None, max: None, step: None }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Point {
    pub x: Box<Expression>,
    pub y: Box<Expression>,
}

#[derive(Serialize, Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Eq,
    Smaller,
    Greater,
    Then,
    Regression,
    Action,
    Condition,
    Range,
    Index,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Pow => "^",
            BinaryOp::Eq => "=",
            BinaryOp::Smaller => "<",
            BinaryOp::Greater => ">",
            BinaryOp::Then => ":",
            BinaryOp::Regression => "~",
            BinaryOp::Action => "->",
            BinaryOp::Range => "...",
            BinaryOp::Index => "",
            _ => panic!("Unexpected display request for {:?}", self)
        };
        write!(f, "{}", s)
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum UnaryOp {
    Plus,
    Minus,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnaryOp::Plus => "+",
            UnaryOp::Minus => "-",
        };
        write!(f, "{}", s)
    }
}


// Folder
#[derive(Serialize, Debug, Clone)]
pub struct Folder {
    pub title: String,
    pub items: Vec<Item>,
}

// Note
#[derive(Serialize, Debug, Clone)]
pub struct Note {
    pub text: String,
}


use crate::tokens::Token;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum LoxLiteral {
    Bool(bool),
    Num(f64),
    Str(String),
    Nil,
}

impl PartialEq for LoxLiteral {
    fn eq(&self, other: &Self) -> bool {
        if self.is_nil() && other.is_nil() {
            return true;
        }

        if self.is_nil() {
            return false;
        }

        if let (Self::Num(left), Self::Num(right)) = (&self, &other) {
            return left == right;
        }

        if let (Self::Str(left), Self::Str(right)) = (&self, &other) {
            return left == right;
        }

        if let (Self::Bool(left), Self::Bool(right)) = (&self, &other) {
            return left == right;
        }

        false
    }
}

impl LoxLiteral {
    pub fn is_bool(&self) -> bool {
        match self {
            Self::Bool(_) => true,
            _ => false,
        }
    }

    pub fn is_num(&self) -> bool {
        match self {
            Self::Num(_) => true,
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        match self {
            Self::Str(_) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Self::Nil => true,
            _ => false,
        }
    }
}

impl<'a> Display for LoxLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxLiteral::Nil => write!(f, "nil"),
            LoxLiteral::Num(val) => write!(f, "{val}"),
            LoxLiteral::Bool(val) => write!(f, "{val}"),
            LoxLiteral::Str(val) => write!(f, "{val}"),
        }
    }
}

pub type Ast = Vec<Stmt>;

#[derive(Debug, Clone)]
pub enum Expr {
    Grouping {
        expr: Box<Expr>,
    },
    Binary {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: Token,
        right: Box<Expr>,
    },
    Literal {
        value: LoxLiteral,
    },
    Variable {
        name: Token,
    },
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expr: Expr,
    },
    Print {
        expr: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
}

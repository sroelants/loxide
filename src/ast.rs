use std::fmt::Display;

use crate::tokens::Token;

#[derive(Debug, Clone)]
pub enum LoxLiteral {
    Bool(bool),
    Num(f64),
    Str(String),
    Nil,
}

impl<'a> Display for LoxLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxLiteral::Nil => write!(f, "nil"),
            LoxLiteral::Num(val) => write!(f, "{val}"),
            LoxLiteral::Bool(val) => write!(f, "{val}"),
            LoxLiteral::Str(val) => {
                let trimmed = std::str::from_utf8(&val.as_bytes()[1..val.len() - 1]).unwrap();
                write!(f, "{trimmed}")
            }
        }
    }
}

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
}

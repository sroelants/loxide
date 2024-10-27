use std::fmt::Display;

use crate::scanner::Token;

pub enum LoxLiteral<'a> {
    Bool(bool),
    Num(f64),
    Str(&'a str),
}

impl<'a> Display for LoxLiteral<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxLiteral::Num(val) => write!(f, "{val}"),
            LoxLiteral::Bool(val) => write!(f, "{val}"),
            LoxLiteral::Str(val) => {
                let trimmed = std::str::from_utf8(&val.as_bytes()[1..val.len() - 1]).unwrap();
                write!(f, "{trimmed}")
            }
        }
    }
}

pub enum Expr<'a> {
    Grouping {
        expr: Box<Expr<'a>>,
    },
    Binary {
        op: Token<'a>,
        left: Box<Expr<'a>>,
        right: Box<Expr<'a>>,
    },
    Unary {
        op: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Literal {
        value: LoxLiteral<'a>,
    },
}

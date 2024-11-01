use crate::{interpreter::Interpreter, tokens::Token};
use std::{fmt::Display, rc::Rc};

#[derive(Clone)]
pub enum LoxLiteral {
    Bool(bool),
    Num(f64),
    Str(String),
    Callable(Rc<dyn Call>), // What goes here?
    Nil,
}

pub trait Call {
    fn call(&self, interpreter: &Interpreter, args: &[LoxLiteral]) -> LoxLiteral;
    fn arity(&self) -> usize;
}

impl Display for dyn Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function>")
    }
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

impl<> Display for LoxLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxLiteral::Nil => write!(f, "nil"),
            LoxLiteral::Num(val) => write!(f, "{val}"),
            LoxLiteral::Bool(val) => write!(f, "{val}"),
            LoxLiteral::Str(val) => write!(f, "{val}"),
            LoxLiteral::Callable(_) => write!(f, "<function>"),
        }
    }
}

pub type Ast = Vec<Stmt>;

#[derive(Clone)]
pub enum Expr {
    Grouping {
        expr: Box<Expr>,
    },
    Binary {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
 },
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>
    },
    Literal {
        value: LoxLiteral,
    },
}

#[derive(Clone)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expr: Expr,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,

    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Print {
        expr: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
}

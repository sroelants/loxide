use crate::class::{Class, Instance};
use crate::functions::{Call, LoxFunction};
use crate::tokens::Token;
use std::hash::Hash;
use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub enum LoxValue {
    Bool(bool),
    Num(f64),
    Str(Rc<String>),
    Nil,
    NativeFunction(Rc<dyn Call>),
    Function(Rc<LoxFunction>),
    Class(Rc<Class>),
    Instance(Instance),
}

impl PartialEq for LoxValue {
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

        if let (Self::Function(left), Self::Function(right)) = (&self, &other) {
            return Rc::ptr_eq(left, right);
        }

        if let (Self::NativeFunction(left), Self::NativeFunction(right)) = (&self, &other) {
            return Rc::ptr_eq(left, right);
        }

        if let (Self::Class(left), Self::Class(right)) = (&self, &other) {
            return Rc::ptr_eq(left, right);
        }

        false
    }
}

impl Eq for LoxValue {}

impl Hash for LoxValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state)
    }
}

impl LoxValue {
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

impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Nil => write!(f, "nil"),
            LoxValue::Num(val) => write!(f, "{val}"),
            LoxValue::Bool(val) => write!(f, "{val}"),
            LoxValue::Str(val) => write!(f, "{val}"),
            LoxValue::Function(val) => write!(f, "{val}"),
            LoxValue::NativeFunction(val) => write!(f, "{val}"),
            LoxValue::Class(val) => write!(f, "{val}"),
            LoxValue::Instance(instance) => write!(f, "{}", instance),
        }
    }
}

pub type Ast = Vec<Stmt>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Grouping {
        expr: Box<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
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
    Set {
        name: Token,
        object: Box<Expr>,
        value: Box<Expr>,
    },
    Logical {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    This {
        keyword: Token,
    },
    Unary {
        op: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Literal {
        value: LoxValue,
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
    Fun {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        expr: Option<Expr>,
    },
    Class {
        name: Token,
        methods: Vec<Stmt>,
    },
}

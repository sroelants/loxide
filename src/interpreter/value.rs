use std::{fmt::Display, rc::Rc};
use std::hash::Hash;

use crate::errors::LoxError;
use crate::span::Spanned;
use crate::syntax::ast::Literal;
use crate::syntax::tokens::Token;
use super::functions::LoxFunction;
use super::functions::Call;
use super::class::{Class, Instance};

#[derive(Debug, Clone)]
pub enum LoxValue {
    Nil,
    Bool(bool),
    Num(f64),
    Str(Rc<String>),
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

    pub fn is_truthy(self: &LoxValue) -> bool {
        match self {
            LoxValue::Nil => false,
            LoxValue::Bool(b) => *b,
            _ => true
        }
    }

    pub fn assert_str(self: LoxValue, op: &Token) -> Result<Rc<String>, Spanned<LoxError>> {
        if let LoxValue::Str(str) = self {
        Ok(str)
        } else {
            Err(Spanned { value: LoxError::TypeError("string"), span: op.span })
        }
    }

    pub fn assert_num(self: LoxValue, op: &Token) -> Result<f64, Spanned<LoxError>> {
        if let LoxValue::Num(num) = self {
        Ok(num)
        } else {
            Err(Spanned { value: LoxError::TypeError("number"), span: op.span })
        }
    }

    pub fn assert_bool(self: LoxValue, op: &Token) -> Result<bool, Spanned<LoxError>> {
        if let LoxValue::Bool(boolean) = self {
        Ok(boolean)
        } else {
            Err(Spanned { value: LoxError::TypeError("bool"), span: op.span })
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

impl From<Literal> for LoxValue {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Nil => Self::Nil,
            Literal::Num(val) => Self::Num(val),
            Literal::Bool(val) => Self::Bool(val),
            Literal::Str(val) => Self::Str(val),
        }
    }
}

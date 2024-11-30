#![allow(dead_code)]
use value::LoxValue;
use environment::Env;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::sourcemap::Source;
use crate::syntax::ast::Ast;
use crate::syntax::ast::Expr;
use crate::span::Spanned;

mod expr;
mod stmt;
mod functions;
mod environment;
mod class;
pub mod resolver;
pub mod value;

type Result<T> = std::result::Result<T, Spanned<RuntimeError>>;
type LoxResult = std::result::Result<LoxValue, Spanned<RuntimeError>>;

pub trait Visitor<T> {
    type Output;
    fn visit(&mut self, node: T) -> Self::Output;
}

pub struct Interpreter<'a> {
    source: &'a Source<'a>,
    pub env: Rc<Env>,
    globals: Rc<Env>,
    locals: HashMap<&'a Expr, usize>,
}

impl<'a> Interpreter<'a> {
    pub fn new(source: &'a Source<'a>, locals: HashMap<&'a Expr, usize>) -> Self {
        let globals = Rc::new(Env::global());

        Self {
            source,
            env: globals.clone(),
            globals,
            locals,
        }
    }

    pub fn push_scope(&mut self) {
        let new_scope =  Env::new(self.env.clone());
        self.env = Rc::new(new_scope);
    }

    pub fn pop_scope(&mut self) {
        self.env = self.env.parent.clone().unwrap();
    }

    pub fn resolve(&mut self, expr: &'a Expr, depth: usize) {
        self.locals.insert(expr, depth);
    }

    pub fn error(&mut self, spanned: Spanned<RuntimeError>) {
        eprintln!("{}", self.source.annotate(spanned));
    }
}

impl<'a> Visitor<&Ast> for Interpreter<'a> {
    type Output = LoxResult;

    fn visit(&mut self, ast: &Ast) -> LoxResult {
        for statement in ast.iter() {
            self.visit(statement)?;
        }

        Ok(LoxValue::Nil)
    }
}

#[derive(Clone)]
pub enum RuntimeError {
    ArityMismatch(usize, usize),
    NotCallable,
    TypeError(&'static str),
    MultiTypeError(&'static str),
    UndeclaredVar(String),
    IllegalPropertyAccess,
    IllegalFieldAccess,
    UndefinedProperty(String),

    // Not actual errors
    Return(LoxValue),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::ArityMismatch(expected, found) => write!(f, "Expected {expected} arguments, but found {found}"),
            RuntimeError::NotCallable => write!(f, "Expression is not callable"),
            RuntimeError::TypeError(ctx) => write!(f, "Operand must be {ctx}"),
            RuntimeError::MultiTypeError(ctx) => write!(f, "Operands must both be {ctx}"),
            RuntimeError::UndeclaredVar(name) => write!(f, "Undeclared variable '{name}'"),
            RuntimeError::IllegalPropertyAccess => write!(f, "Only class instances have properties"),
            RuntimeError::IllegalFieldAccess => write!(f, "Only class instances have fields"),
            RuntimeError::UndefinedProperty(name) => write!(f, "Undefined property '{name}'"),

            // Not an actual error, should never make it to the error reporting
            // stage
            RuntimeError::Return(_) => unreachable!()
        }
    }
}

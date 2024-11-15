#![allow(dead_code)]
use value::LoxValue;
use environment::Env;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Ast;
use crate::ast::Expr;
use crate::errors::LoxError;
use crate::span::Spanned;

mod expr;
mod stmt;
mod functions;
mod environment;
mod class;
pub mod resolver;
pub mod value;

type Result<T> = std::result::Result<T, Spanned<LoxError>>;
type LoxResult = std::result::Result<LoxValue, Spanned<LoxError>>;

pub trait Visitor<T> {
    fn visit(&mut self, node: &T) -> LoxResult;
}

pub struct Interpreter<'a> {
    pub env: Rc<Env>,
    globals: Rc<Env>,
    locals: HashMap<&'a Expr, usize>,

}

impl<'a> Interpreter<'a> {
    pub fn new(locals: HashMap<&'a Expr, usize>) -> Self {
        let globals = Rc::new(Env::global());

        Self {
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
}

impl<'a> Visitor<Ast> for Interpreter<'a> {
    fn visit(&mut self, ast: &Ast) -> LoxResult {
        for statement in ast.iter() {
            self.visit(statement)?;
        }

        Ok(LoxValue::Nil)
    }
}

use std::{collections::HashMap, rc::Rc};

use crate::errors::LoxError;
use crate::functions::globals::Clock;
use crate::span::Spanned;
use crate::tokens::Token;
use crate::ast::LoxLiteral;

pub struct Env {
    scopes: Vec<HashMap<String, LoxLiteral>>,
}

impl Env {
    pub fn new() -> Self {
        let mut root_scope = HashMap::new();
        root_scope.insert(format!("clock"), LoxLiteral::Callable(Rc::new(Clock)));

        Self {
            scopes: vec![root_scope],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: &Token, value: LoxLiteral) {
        self.scopes.last_mut().unwrap().insert(name.lexeme.to_owned(), value);
    }

    pub fn assign(&mut self, name: &Token, value: LoxLiteral) -> Result<(), Spanned<LoxError>> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name.lexeme) {
                scope.insert(name.lexeme.to_owned(), value);
                return Ok(())
            }
        }

        Err(Spanned {
            value: LoxError::UndeclaredVar(format!("{name}")),
            span: name.span
        })
    }

    pub fn get(&self, name: &Token) -> Result<LoxLiteral, Spanned<LoxError>> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(&name.lexeme) {
                return Ok(value.to_owned());
            }
        }

        Err(Spanned {
            value: LoxError::UndeclaredVar(format!("{name}")),
            span: name.span
        })
    }
}

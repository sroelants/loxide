use std::collections::HashMap;

use crate::{ast::LoxLiteral, interpreter::RuntimeError, tokens::Token};

pub struct Env {
    scopes: Vec<HashMap<String, LoxLiteral>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: Token, value: LoxLiteral) {
        self.scopes.last_mut().unwrap().insert(name.lexeme, value);
    }

    pub fn assign(&mut self, name: Token, value: LoxLiteral) -> Result<(), RuntimeError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name.lexeme) {
                scope.insert(name.lexeme, value);
                return Ok(())
            }
        }

        Err(RuntimeError {
            msg: format!("undeclared variable '{name}'"),
            token: name,
        })
    }

    pub fn get(&self, name: Token) -> Result<LoxLiteral, RuntimeError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(&name.lexeme) {
                return Ok(value.to_owned());
            }
        }

        Err(RuntimeError {
            msg: format!("undeclared variable '{name}'"),
            token: name,
        })
    }
}

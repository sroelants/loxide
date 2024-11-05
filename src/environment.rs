use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::errors::LoxError;
use crate::functions::globals::Clock;
use crate::span::Spanned;
use crate::tokens::Token;
use crate::ast::LoxLiteral;

type Bindings = HashMap<String, LoxLiteral>;

pub struct Env {
    pub parent: Option<Rc<Env>>,
    pub bindings: Rc<RefCell<Bindings>>,
}

impl Env {
    pub fn new(parent: Rc<Env>) -> Self {
        Self {
            parent: Some(parent),
            bindings: Rc::new(RefCell::new(Bindings::new())),
        }
    }

    pub fn global() -> Self {
        let mut bindings = Bindings::new();
        bindings.insert(format!("clock"), LoxLiteral::Callable(Rc::new(Clock)));

        Self {
            parent: None,
            bindings: Rc::new(RefCell::new(bindings))
        }
    }

    pub fn define(&self, name: &Token, value: LoxLiteral) {
        self.bindings.borrow_mut().insert(name.lexeme.to_owned(), value);
    }

    pub fn assign(&self, name: &Token, value: LoxLiteral) -> Result<(), Spanned<LoxError>> {
        if RefCell::borrow(&self.bindings).contains_key(&name.lexeme) {
            self.bindings.borrow_mut().insert(name.lexeme.to_owned(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.assign(name, value)
        } else {
            Err(Spanned {
                value: LoxError::UndeclaredVar(format!("{name}")),
                span: name.span
            })
        }
    }

    pub fn get(&self, name: &Token) -> Result<LoxLiteral, Spanned<LoxError>> {
        if let Some(value) = RefCell::borrow(&self.bindings).get(&name.lexeme) {
            Ok(value.to_owned())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            Err(Spanned {
                value: LoxError::UndeclaredVar(format!("{name}")),
                span: name.span
            })
        }
    }

    pub fn get_at(&self, dist: usize, name: &Token) -> Result<LoxLiteral, Spanned<LoxError>> {
        let mut env = self;

        for _ in 0..dist {
            env = env.parent.as_ref().unwrap().borrow();
        }

        env.get(name)
    }

    pub fn assign_at(
        &self,
        dist: usize,
        name: &Token,
        value: LoxLiteral
    ) -> Result<(), Spanned<LoxError>> {
        let mut env = self;

        for _ in 0..dist {
            env = env.parent.as_ref().unwrap().borrow();
        }

        env.assign(name, value)
    }
}

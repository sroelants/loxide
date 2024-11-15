use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::errors::LoxError;
use super::functions::globals::Clock;
use crate::span::Spanned;
use crate::tokens::Token;
use crate::interpreter::value::LoxValue;

type Bindings = HashMap<String, LoxValue>;

#[derive(Debug)]
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
        bindings.insert(format!("clock"), LoxValue::NativeFunction(Rc::new(Clock)));

        Self {
            parent: None,
            bindings: Rc::new(RefCell::new(bindings))
        }
    }

    pub fn define(&self, name: String, value: LoxValue) {
        self.bindings.borrow_mut().insert(name, value);
    }

    pub fn assign(&self, name: &Token, value: LoxValue) -> Result<(), Spanned<LoxError>> {
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

    pub fn get(&self, name: &Token) -> Result<LoxValue, Spanned<LoxError>> {
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

    pub fn get_at(&self, dist: usize, name: &Token) -> Result<LoxValue, Spanned<LoxError>> {
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
        value: LoxValue
    ) -> Result<(), Spanned<LoxError>> {
        let mut env = self;

        for _ in 0..dist {
            env = env.parent.as_ref().unwrap().borrow();
        }

        env.assign(name, value)
    }
}

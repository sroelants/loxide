use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{ast::{Call, LoxLiteral}, errors::{BaseError, Stage}, interpreter::Interpreter, tokens::Token};

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

    pub fn assign(&mut self, name: &Token, value: LoxLiteral) -> Result<(), BaseError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name.lexeme) {
                scope.insert(name.lexeme.to_owned(), value);
                return Ok(())
            }
        }

        Err(BaseError {
            stage: Stage::Runtime,
            msg: format!("undeclared variable '{name}'"),
            span: name.span,
        })
    }

    pub fn get(&self, name: &Token) -> Result<LoxLiteral, BaseError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(&name.lexeme) {
                return Ok(value.to_owned());
            }
        }

        Err(BaseError {
            stage: Stage::Runtime,
            msg: format!("undeclared variable '{name}'"),
            span: name.span,
        })
    }
}

// Globals
struct Clock;

impl Call for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &Interpreter, _args: &[LoxLiteral]) -> LoxLiteral {
        use std::time::{SystemTime, UNIX_EPOCH};

        let epoch_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        LoxLiteral::Num(epoch_millis / 1000.0)
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn: clock>")
    }
}

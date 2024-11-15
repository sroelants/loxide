use std::{fmt::Display, rc::Rc};
use std::fmt::Debug;

use super::environment::Env;
use super::class::Instance;
use crate::interpreter::value::LoxValue;
use crate::syntax::tokens::Token;
use crate::span::Spanned;
use crate::interpreter::Interpreter;
use crate::errors::LoxError;
use crate::syntax::ast::Stmt;

pub trait Call: Display + Debug {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[LoxValue],
    ) -> Result<LoxValue, Spanned<LoxError>>;

    fn arity(&self) -> usize;
}

#[derive(Clone)]
pub struct LoxFunction {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub env: Rc<Env>,
}

impl Call for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[LoxValue],
    ) -> Result<LoxValue, Spanned<LoxError>> {
        let local_scope = Rc::new(Env::new(self.env.clone()));

        for (param, arg) in self.params.iter().zip(args) {
            local_scope.define(param.lexeme.clone(), arg.clone())
        }

        // Catch any return statements that are bubbled up by throwing an error
        match interpreter.exec_block_with_env(&self.body, local_scope) {
            Err(Spanned {
                value: LoxError::Return(value),
                ..
            }) => Ok(value),

            // Just pass through anything else
            val => val,
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl LoxFunction {
    pub fn bind(mut self, instance: &Instance) -> LoxFunction {
        self.env = Rc::new(Env::new(self.env));
        self.env.define(format!("this"), LoxValue::Instance(instance.clone()));
        self
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function {name}>", name = self.name)
    }
}

impl Debug for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoxFunction").field("name", &self.name).field("params", &self.params).field("body", &self.body).finish()
    }
}

// Globals
pub mod globals {
    use std::fmt::Display;
    use std::fmt::Debug;

    use crate::interpreter::value::LoxValue;
    use crate::{errors::LoxError, interpreter::Interpreter, span::Spanned};

    use super::Call;

    pub struct Clock;

    impl Call for Clock {
        fn arity(&self) -> usize {
            0
        }

        fn call(
            &self,
            _interpreter: &mut Interpreter,
            _args: &[LoxValue],
        ) -> Result<LoxValue, Spanned<LoxError>> {
            use std::time::{SystemTime, UNIX_EPOCH};

            let epoch_millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as f64;

            Ok(LoxValue::Num(epoch_millis / 1000.0))
        }
    }

    impl Display for Clock {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "<native fn: clock>")
        }
    }

    impl Debug for Clock {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Clock").finish()
        }
    }
}

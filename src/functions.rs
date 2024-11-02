use std::{fmt::Display, rc::Rc};

use crate::{
    ast::{LoxLiteral, Stmt},
    environment::Env,
    errors::LoxError,
    interpreter::Interpreter,
    span::Spanned,
    tokens::Token,
};

pub trait Call: Display {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[LoxLiteral],
    ) -> Result<LoxLiteral, Spanned<LoxError>>;

    fn arity(&self) -> usize;
}

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
        args: &[LoxLiteral],
    ) -> Result<LoxLiteral, Spanned<LoxError>> {
        let local_scope = Rc::new(Env::new(self.env.clone()));

        for (param, arg) in self.params.iter().zip(args) {
            local_scope.define(param, arg.clone())
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

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function {name}>", name = self.name)
    }
}

// Globals
pub mod globals {
    use std::fmt::Display;

    use crate::{ast::LoxLiteral, errors::LoxError, interpreter::Interpreter, span::Spanned};

    use super::Call;

    pub struct Clock;

    impl Call for Clock {
        fn arity(&self) -> usize {
            0
        }

        fn call(
            &self,
            _interpreter: &mut Interpreter,
            _args: &[LoxLiteral],
        ) -> Result<LoxLiteral, Spanned<LoxError>> {
            use std::time::{SystemTime, UNIX_EPOCH};

            let epoch_millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as f64;

            Ok(LoxLiteral::Num(epoch_millis / 1000.0))
        }
    }

    impl Display for Clock {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "<native fn: clock>")
        }
    }
}

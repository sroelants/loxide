use std::fmt::Display;

use crate::{functions::Call, tokens::Token};

#[derive(Debug)]
pub struct Class {
    pub name: Token,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Call for Class {
    fn call(
        &self,
        _interpreter: &mut crate::interpreter::Interpreter,
        _args: &[crate::ast::LoxLiteral],
    ) -> Result<crate::ast::LoxLiteral, crate::span::Spanned<crate::errors::LoxError>> {
        todo!()
    }

    fn arity(&self) -> usize {
        todo!()
    }
}

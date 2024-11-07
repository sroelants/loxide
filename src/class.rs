use std::collections::HashMap;
use std::fmt::Display;
use crate::ast::LoxLiteral;
use crate::span::Spanned;
use crate::errors::LoxError;
use crate::interpreter::Interpreter;

use crate::{functions::Call, tokens::Token};

#[derive(Debug, Clone)]
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
        _interpreter: &mut Interpreter,
        _args: &[LoxLiteral],
    ) -> Result<LoxLiteral, Spanned<LoxError>> {
        let instance = Instance {
            class: self.clone(),
            fields: HashMap::new(),
        };

        Ok(LoxLiteral::Instance(instance))
    }

    fn arity(&self) -> usize {
        return 0;
    }
}

// FIXME: Instances should store a _pointer_ to the class, so that I can mutate
// the class at runtime, and all instances will have access to the new, mutated,
// class properties. Note that the _instance_ is the thing that stores the state,
// though!
#[derive(Debug, Clone)]
pub struct Instance {
    class: Class,
    fields: HashMap<String, LoxLiteral>
}

impl Instance {
    pub fn get(&self, name: &Token) -> Result<&LoxLiteral, Spanned<LoxError>> {
        if let Some(value) = self.fields.get(&name.lexeme) {
            Ok(value)
        } else {
            Err(Spanned {
                value: LoxError::UndefinedProperty(name.lexeme.clone()),
                span: name.span
            })
        }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.class)
    }
}

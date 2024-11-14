
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use crate::functions::LoxFunction;
use crate::span::Spanned;
use crate::errors::LoxError;
use crate::interpreter::{Interpreter, LoxValue};

use crate::{functions::Call, tokens::Token};

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Token,
    pub methods: HashMap<String, Rc<LoxFunction>>
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Call for Rc<Class> {
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _args: &[LoxValue],
    ) -> Result<LoxValue, Spanned<LoxError>> {
        let instance = Instance(Rc::new(RefCell::new(InstanceInner {
            class: self.clone(),
            fields: HashMap::new(),
        })));

        Ok(LoxValue::Instance(instance))
    }

    fn arity(&self) -> usize {
        return 0;
    }
}

#[derive(Debug, Clone)]
pub struct InstanceInner {
    pub class: Rc<Class>,
    pub fields: HashMap<String, LoxValue>
}

#[derive(Debug, Clone)]
pub struct Instance(pub Rc<RefCell<InstanceInner>>);


impl Instance {
    pub fn get(&self, name: &Token) -> Result<LoxValue, Spanned<LoxError>> {
        if let Some(value) = self.0.borrow().fields.get(&name.lexeme) {
            Ok(value.to_owned())
        } else if let Some(method) = self.0.borrow().class.methods.get(&name.lexeme) {
            Ok(LoxValue::Function(Rc::new(Rc::unwrap_or_clone(method.clone()).bind(&self.clone()))))
        } else {
            Err(Spanned {
                value: LoxError::UndefinedProperty(name.lexeme.clone()),
                span: name.span
            })
        }
    }

    pub fn set(&mut self, name: &Token, value: LoxValue) {
        self.0.borrow_mut().fields.insert(name.lexeme.clone(), value);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.borrow().class)
    }
}

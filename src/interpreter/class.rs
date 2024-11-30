use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use super::functions::LoxFunction;
use super::functions::Call;
use super::RuntimeError;
use crate::span::Spanned;
use crate::interpreter::Interpreter;
use crate::interpreter::value::LoxValue;
use crate::syntax::tokens::Token;

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
    ) -> Result<LoxValue, Spanned<RuntimeError>> {
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
    pub fn get(&self, name: &Token) -> Result<LoxValue, Spanned<RuntimeError>> {
        if let Some(value) = self.0.borrow().fields.get(&name.lexeme) {
            Ok(value.to_owned())
        } else if let Some(method) = self.0.borrow().class.methods.get(&name.lexeme) {
            Ok(LoxValue::Function(Rc::new(Rc::unwrap_or_clone(method.clone()).bind(&self.clone()))))
        } else {
            Err(Spanned {
                value: RuntimeError::UndefinedProperty(name.lexeme.clone()),
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

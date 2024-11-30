use std::{collections::HashMap, rc::Rc};

use crate::syntax::ast::Stmt;
use crate::span::Spanned;
use crate::span::Span;
use super::functions::LoxFunction;
use super::environment::Env;
use super::class::Class;
use super::value::LoxValue;

use super::{Interpreter, LoxResult, Visitor, RuntimeError};

impl<'a> Visitor<&Stmt> for Interpreter<'a> {
    type Output = LoxResult;

    fn visit(&mut self, statement: &Stmt) -> LoxResult {
        match statement {
            Stmt::Print { expr } => {
                let val = self.evaluate(expr)?;
                println!("{val}");
            }

            Stmt::Return { expr, .. } => {
                let value = if let Some(expr) = expr {
                    self.evaluate(expr)?
                } else {
                    LoxValue::Nil
                };

                Err(Spanned {
                    value: RuntimeError::Return(value),
                    span: Span::new(),
                })?;
            }

            Stmt::If { condition, then_branch, else_branch } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }

            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
            }

            Stmt::Expression { expr } => {
               self.evaluate(expr)?;
            }

            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    LoxValue::Nil
                };

                self.env.define(name.lexeme.clone(), value);
            }

            Stmt::Block { statements } => {
                self.exec_block(statements)?;
            }

            Stmt::Fun { name, params, body } => {
                let function = LoxFunction {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    env: self.env.clone(),
                };

                self.env.define(name.lexeme.clone(), LoxValue::Function(Rc::new(function)));
            },

            Stmt::Class { name, methods } => {
                self.env.define(name.lexeme.clone(), LoxValue::Nil);

                let mut methods_map = HashMap::new();

                for method in methods {
                    let Stmt::Fun { name, params, body } = method else { panic!() };

                    let function = LoxFunction {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                        env: self.env.clone(),
                    };

                    methods_map.insert(name.lexeme.clone(), Rc::new(function));
                }

                let class = Class { name: name.clone(), methods: methods_map };
                self.env.assign(name, LoxValue::Class(Rc::new(class)))?;
            }
        };

        Ok(LoxValue::Nil)
    }

}

impl<'a> Interpreter<'a> {
    fn execute(&mut self, statement: &Stmt) -> LoxResult {
        self.visit(statement)
    }

    fn exec_block(&mut self, statements: &Vec<Stmt>) -> LoxResult {
        self.push_scope();

        for statement in statements.iter() {
            if let Err(err) = self.execute(statement) {
                self.pop_scope();
                return Err(err);
            }
        }

        self.pop_scope();
        Ok(LoxValue::Nil)
    }

    // Additional helper that allows us to execute a block with a given environment.
    pub fn exec_block_with_env(&mut self, statements: &Vec<Stmt>, env: Rc<Env>) -> LoxResult {
        let prev_env = std::mem::replace(&mut self.env, env);

        for statement in statements.iter() {
            if let Err(err) = self.execute(statement) {
                self.env = prev_env;
                return Err(err);
            }
        }

        self.env = prev_env;
        Ok(LoxValue::Nil)
    }
}

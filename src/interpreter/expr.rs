#![allow(dead_code)]
use std::rc::Rc;

use crate::interpreter::LoxValue as Val;
use crate::ast::Expr;
use crate::errors::LoxError;
use super::functions::Call;
use crate::span::Spanned;
use crate::tokens::Token;
use crate::tokens::TokenType;

use super::{Interpreter, LoxResult, Visitor};

impl<'a> Visitor<Expr> for Interpreter<'a> {
    fn visit(&mut self, expr: &Expr) -> LoxResult {
        match expr {
            Expr::Literal { value } => Ok(value.clone().into()),

            Expr::Call { callee, arguments, paren } => self.visit_call(callee, arguments, &paren),

            Expr::Grouping { expr } => self.visit(expr.as_ref()),

            Expr::Unary { op, right } => self.visit_unary(op, right),

            Expr::Binary { op, left, right } => self.visit_binary(op, left, right),

            Expr::Logical { op, left, right } => self.visit_logical(op, left, right),

            Expr::Variable { name } => { self.lookup(name, expr)},

            Expr::Assignment { name, value } => {
                let value = self.evaluate(value)?;

                if let Some(distance) = self.locals.get(expr) {
                    self.env.assign_at(*distance, name, value.clone())?;
                } else {
                    self.globals.assign(name, value.clone())?;
                }

                Ok(value)
            },

            Expr::Get { name, object } => {
                let object = self.evaluate(object)?;

                if let Val::Instance(instance) = object {
                    instance.get(name)
                } else {
                    Err(Spanned {
                        value: LoxError::IllegalPropertyAccess,
                        span: name.span
                    })

                }
            },

            Expr::Set { name, value, object } => {
                let object = self.evaluate(object)?;

                if let Val::Instance(mut instance) = object {
                    let value = self.evaluate(value)?;
                    instance.set(name, value.clone());
                    Ok(value)
                } else {
                    Err(Spanned {
                        value: LoxError::IllegalFieldAccess,
                        span: name.span,
                    })
                }
            },

            Expr::This { keyword } => {
                self.lookup(keyword, expr)
            },
        }
    }
}

impl<'a> Interpreter<'a> {
    pub fn evaluate(&mut self, expr: &Expr) -> LoxResult {
        self.visit(expr)
    }

    fn lookup(&self, name: &Token, expr: &Expr) -> LoxResult {
        if let Some(&dist) = self.locals.get(expr) {
            self.env.get_at(dist, name)
        } else {
            self.globals.get(name)
        }
    }

    fn visit_call(&mut self, callee: &Expr, args: &[Expr], token: &Token) -> LoxResult {
        let callee = self.evaluate(callee)?;
        let mut evaluated_args = Vec::new();

        for arg in args {
            evaluated_args.push(self.evaluate(arg)?);
        }

        match callee {
            Val::NativeFunction(fun) => {
                if args.len() != fun.arity() {
                    return Err(Spanned {
                        value: LoxError::ArityMismatch(fun.arity(), args.len()),
                        span: token.span,
                    });
                }

                fun.call(self, &evaluated_args)
            },
            Val::Function(fun) => {
                if args.len() != fun.arity() {
                    return Err(Spanned {
                        value: LoxError::ArityMismatch(fun.arity(), args.len()),
                        span: token.span,
                    });
                }

                fun.call(self, &evaluated_args)
            },
            Val::Class(fun) => {
                if args.len() != fun.arity() {
                    return Err(Spanned {
                        value: LoxError::ArityMismatch(fun.arity(), args.len()),
                        span: token.span,
                    });
                }

                fun.call(self, &evaluated_args)
            },
            _ => {
                Err(Spanned {
                    value: LoxError::NotCallable,
                    span: token.span,
                })
            }
        }
    }

    fn visit_unary(&mut self, op: &Token, right: &Expr) -> LoxResult {
        let right = self.evaluate(right)?;

        match op.token_type {
            TokenType::Bang => Ok(Val::Bool(!right.is_truthy())),

            TokenType::Minus => {
                let num = right.assert_num(&op)?;
                Ok(Val::Num(-num))
            },

            _ => unreachable!(),
        }
    }

    fn visit_logical(&mut self, op: &Token, left: &Expr, right: &Expr) -> LoxResult {
        let left = self.evaluate(left)?;

        if op.token_type == TokenType::Or {
            if left.is_truthy() {
                return Ok(left);
            }
        } else {
            if !left.is_truthy() {
                return Ok(left);
            }
        }

        self.evaluate(right)
    }

    fn visit_binary(&mut self, op: &Token, left: &Expr, right: &Expr) -> LoxResult {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match op.token_type {
            TokenType::Minus => {
                let left = left.assert_num(&op)?;
                let right = right.assert_num(&op)?;

                Ok(Val::Num(left - right))
            },

            TokenType::Plus => {
                if let (Val::Num(left), Val::Num(right)) = (&left, &right) {
                    Ok(Val::Num(left + right))
                } else if let (Val::Str(left), Val::Str(right)) = (left, right) {
                    Ok(Val::Str(Rc::new(format!("{left}{right}"))))
                } else {
                    Err(Spanned {
                        value: LoxError::MultiTypeError("string or number"),
                        span: op.span,
                    })
                }
            }

            TokenType::Star => {
                let left = left.assert_num(&op)?;
                let right = right.assert_num(&op)?;

                Ok(Val::Num(left * right))
            },

            TokenType::Slash => {
                let left = left.assert_num(&op)?;
                let right = right.assert_num(&op)?;

                Ok(Val::Num(left / right))

            },

            TokenType::Greater => {
                let left = left.assert_num(&op)?;
                let right = right.assert_num(&op)?;

                Ok(Val::Bool(left > right))
            },

            TokenType::GreaterEqual => {
                let left = left.assert_num(&op)?;
                let right = right.assert_num(&op)?;

                Ok(Val::Bool(left >= right))
            },

            TokenType::Less => {
                let left = left.assert_num(&op)?;
                let right = right.assert_num(&op)?;

                Ok(Val::Bool(left < right))
            },

            TokenType::LessEqual => {
                let left = left.assert_num(&op)?;
                let right = right.assert_num(&op)?;

                Ok(Val::Bool(left <= right))
            },

            TokenType::BangEqual => Ok(Val::Bool(left != right)),

            TokenType::EqualEqual => Ok(Val::Bool(left == right)),

            _ => unreachable!()
        }
    }
}

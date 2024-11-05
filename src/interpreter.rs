#![allow(dead_code)]
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Ast;
use crate::ast::LoxLiteral as Lit;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::environment::Env;
use crate::errors::LoxError;
use crate::functions::LoxFunction;
use crate::span::Span;
use crate::span::Spanned;
use crate::tokens::Token;
use crate::tokens::TokenType;
use crate::util::RefEq;

type Result<T> = std::result::Result<T, Spanned<LoxError>>;

pub struct Interpreter<'a> {
    pub env: Rc<Env>,
    globals: Rc<Env>,
    locals: HashMap<RefEq<'a, Expr>, usize>,

}

impl<'a> Interpreter<'a> {
    pub fn new(locals: HashMap<RefEq<'a, Expr>, usize>) -> Self {
        let globals = Rc::new(Env::global());

        Self {
            env: globals.clone(),
            globals,
            locals,
        }
    }

    pub fn push_scope(&mut self) {
        let new_scope =  Env::new(self.env.clone());
        self.env = Rc::new(new_scope);
    }

    pub fn pop_scope(&mut self) {
        self.env = self.env.parent.clone().unwrap();
    }

    pub fn resolve(&mut self, expr: &'a Expr, depth: usize) {
        self.locals.insert(RefEq(expr), depth);
    }

    pub fn interpret(&mut self, ast: &Ast) -> Result<Lit> {
        for statement in ast.iter() {
            self.execute(statement)?;
        }

        Ok(Lit::Nil)
    }

    fn execute(&mut self, statement: &Stmt) -> Result<Lit> {
        match statement {
            Stmt::Print { expr } => {
                let val = self.evaluate(expr)?;
                println!("{val}");
            }

            Stmt::Return { expr, .. } => {
                let value = if let Some(expr) = expr {
                    self.evaluate(expr)?
                } else {
                    Lit::Nil
                };

                Err(Spanned {
                    value: LoxError::Return(value),
                    span: Span::new(),
                })?;
            }

            Stmt::If { condition, then_branch, else_branch } => {
                if is_truthy(&self.evaluate(condition)?) {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }

            Stmt::While { condition, body } => {
                while is_truthy(&self.evaluate(condition)?) {
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
                    Lit::Nil
                };

                self.env.define(name, value);
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

                self.env.define(name, Lit::Callable(Rc::new(function)));
            },
        };

        Ok(Lit::Nil)
    }

    fn exec_block(&mut self, statements: &Vec<Stmt>) -> Result<Lit> {
        self.push_scope();

        for statement in statements.iter() {
            if let Err(err) = self.execute(statement) {
                self.pop_scope();
                return Err(err);
            }
        }

        self.pop_scope();
        Ok(Lit::Nil)
    }

    // Additional helper that allows us to execute a block with a given environment.
    pub fn exec_block_with_env(&mut self, statements: &Vec<Stmt>, env: Rc<Env>) -> Result<Lit> {
        let prev_env = std::mem::replace(&mut self.env, env);

        for statement in statements.iter() {
            if let Err(err) = self.execute(statement) {
                self.env = prev_env;
                return Err(err);
            }
        }

        self.env = prev_env;
        Ok(Lit::Nil)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Lit> {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),

            Expr::Call { callee, arguments, paren } => self.eval_call(callee, arguments, &paren),

            Expr::Grouping { expr } => self.evaluate(expr),

            Expr::Unary { op, right } => self.eval_unary(op, right),

            Expr::Binary { op, left, right } => self.eval_binary(op, left, right),

            Expr::Logical { op, left, right } => self.eval_logical(op, left, right),

            Expr::Variable { name } => { self.lookup(name, expr)},

            Expr::Assignment { name, value } => {
                let value = self.evaluate(value)?;

                if let Some(distance) = self.locals.get(&RefEq(expr)) {
                    self.env.assign_at(*distance, name, value.clone())?;
                } else {
                    self.globals.assign(name, value.clone())?;
                }

                Ok(value)
            }
        }
    }

    fn lookup(&self, name: &Token, expr: &Expr) -> Result<Lit> {
        if let Some(&dist) = self.locals.get(&RefEq(expr)) {
            self.env.get_at(dist, name)
        } else {
            Err(Spanned {
                span: name.span,
                value: LoxError::UndeclaredVar(name.lexeme.to_owned()),
            })
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr], token: &Token) -> Result<Lit> {
        let callee = self.evaluate(callee)?;
        let mut evaluated_args = Vec::new();

        for arg in args {
            evaluated_args.push(self.evaluate(arg)?);
        }

        if let Lit::Callable(fun) = callee {
            if args.len() != fun.arity() {
                return Err(Spanned {
                    value: LoxError::ArityMismatch(fun.arity(), args.len()),
                    span: token.span,
                });
            }

            fun.call(self, &evaluated_args)
        } else {
            Err(Spanned {
                value: LoxError::NotCallable,
                span: token.span,
            })
        }
    }

    fn eval_unary(&mut self, op: &Token, right: &Expr) -> Result<Lit> {
        let right = self.evaluate(right)?;

        match op.token_type {
            TokenType::Bang => Ok(Lit::Bool(!is_truthy(&right))),

            TokenType::Minus => {
                let num = assert_num(&op, right)?;
                Ok(Lit::Num(-num))
            },

            _ => unreachable!(),
        }
    }

    fn eval_logical(&mut self, op: &Token, left: &Expr, right: &Expr) -> Result<Lit> {
        let left = self.evaluate(left)?;

        if op.token_type == TokenType::Or {
            if is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !is_truthy(&left) {
                return Ok(left);
            }
        }

        self.evaluate(right)
    }

    fn eval_binary(&mut self, op: &Token, left: &Expr, right: &Expr) -> Result<Lit> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match op.token_type {
            TokenType::Minus => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Lit::Num(left - right))
            },

            TokenType::Plus => {
                if let (Lit::Num(left), Lit::Num(right)) = (&left, &right) {
                    Ok(Lit::Num(left + right))
                } else if let (Lit::Str(left), Lit::Str(right)) = (left, right) {
                    Ok(Lit::Str(format!("{left}{right}")))
                } else {
                    Err(Spanned {
                        value: LoxError::MultiTypeError("string or number"),
                        span: op.span,
                    })
                }
            }

            TokenType::Star => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Lit::Num(left * right))
            },

            TokenType::Slash => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Lit::Num(left / right))

            },

            TokenType::Greater => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Lit::Bool(left > right))
            },

            TokenType::GreaterEqual => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Lit::Bool(left >= right))
            },

            TokenType::Less => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Lit::Bool(left < right))
            },

            TokenType::LessEqual => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Lit::Bool(left <= right))
            },

            TokenType::BangEqual => Ok(Lit::Bool(left != right)),

            TokenType::EqualEqual => Ok(Lit::Bool(left == right)),

            _ => unreachable!()
        }
    }
}

fn is_truthy(value: &Lit) -> bool {
    match value {
        Lit::Nil => false,
        Lit::Bool(b) => *b,
        _ => true
    }
}

fn assert_str(op: &Token, lit: Lit) -> Result<String> {
    if let Lit::Str(str) = lit {
       Ok(str)
    } else {
        Err(Spanned { value: LoxError::TypeError("string"), span: op.span })
    }
}

fn assert_num(op: &Token, lit: Lit) -> Result<f64> {
    if let Lit::Num(num) = lit {
       Ok(num)
    } else {
        Err(Spanned { value: LoxError::TypeError("number"), span: op.span })
    }
}

fn assert_bool(op: &Token, lit: Lit) -> Result<bool> {
    if let Lit::Bool(boolean) = lit {
       Ok(boolean)
    } else {
        Err(Spanned { value: LoxError::TypeError("bool"), span: op.span })
    }
}

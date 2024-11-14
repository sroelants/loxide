#![allow(dead_code)]
use std::fmt::Display;
use std::hash::Hash;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Ast;
use crate::ast::Literal;
use crate::class::Instance;
use crate::interpreter::LoxValue as Val;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::class::Class;
use crate::environment::Env;
use crate::errors::LoxError;
use crate::functions::Call;
use crate::functions::LoxFunction;
use crate::span::Span;
use crate::span::Spanned;
use crate::tokens::Token;
use crate::tokens::TokenType;

type Result<T> = std::result::Result<T, Spanned<LoxError>>;

pub struct Interpreter<'a> {
    pub env: Rc<Env>,
    globals: Rc<Env>,
    locals: HashMap<&'a Expr, usize>,

}

impl<'a> Interpreter<'a> {
    pub fn new(locals: HashMap<&'a Expr, usize>) -> Self {
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
        self.locals.insert(expr, depth);
    }

    pub fn interpret(&mut self, ast: &Ast) -> Result<Val> {
        for statement in ast.iter() {
            self.execute(statement)?;
        }

        Ok(Val::Nil)
    }

    fn execute(&mut self, statement: &Stmt) -> Result<Val> {
        match statement {
            Stmt::Print { expr } => {
                let val = self.evaluate(expr)?;
                println!("{val}");
            }

            Stmt::Return { expr, .. } => {
                let value = if let Some(expr) = expr {
                    self.evaluate(expr)?
                } else {
                    Val::Nil
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
                    Val::Nil
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

                self.env.define(name.lexeme.clone(), Val::Function(Rc::new(function)));
            },

            Stmt::Class { name, methods } => {
                self.env.define(name.lexeme.clone(), Val::Nil);

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
                self.env.assign(name, Val::Class(Rc::new(class)))?;
            }
        };

        Ok(Val::Nil)
    }

    fn exec_block(&mut self, statements: &Vec<Stmt>) -> Result<Val> {
        self.push_scope();

        for statement in statements.iter() {
            if let Err(err) = self.execute(statement) {
                self.pop_scope();
                return Err(err);
            }
        }

        self.pop_scope();
        Ok(Val::Nil)
    }

    // Additional helper that allows us to execute a block with a given environment.
    pub fn exec_block_with_env(&mut self, statements: &Vec<Stmt>, env: Rc<Env>) -> Result<Val> {
        let prev_env = std::mem::replace(&mut self.env, env);

        for statement in statements.iter() {
            if let Err(err) = self.execute(statement) {
                self.env = prev_env;
                return Err(err);
            }
        }

        self.env = prev_env;
        Ok(Val::Nil)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Val> {
        match expr {
            Expr::Literal { value } => Ok(value.clone().into()),

            Expr::Call { callee, arguments, paren } => self.eval_call(callee, arguments, &paren),

            Expr::Grouping { expr } => self.evaluate(expr),

            Expr::Unary { op, right } => self.eval_unary(op, right),

            Expr::Binary { op, left, right } => self.eval_binary(op, left, right),

            Expr::Logical { op, left, right } => self.eval_logical(op, left, right),

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

    fn lookup(&self, name: &Token, expr: &Expr) -> Result<Val> {
        if let Some(&dist) = self.locals.get(expr) {
            self.env.get_at(dist, name)
        } else {
            self.globals.get(name)
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr], token: &Token) -> Result<Val> {
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

    fn eval_unary(&mut self, op: &Token, right: &Expr) -> Result<Val> {
        let right = self.evaluate(right)?;

        match op.token_type {
            TokenType::Bang => Ok(Val::Bool(!is_truthy(&right))),

            TokenType::Minus => {
                let num = assert_num(&op, right)?;
                Ok(Val::Num(-num))
            },

            _ => unreachable!(),
        }
    }

    fn eval_logical(&mut self, op: &Token, left: &Expr, right: &Expr) -> Result<Val> {
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

    fn eval_binary(&mut self, op: &Token, left: &Expr, right: &Expr) -> Result<Val> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match op.token_type {
            TokenType::Minus => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

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
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Val::Num(left * right))
            },

            TokenType::Slash => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Val::Num(left / right))

            },

            TokenType::Greater => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Val::Bool(left > right))
            },

            TokenType::GreaterEqual => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Val::Bool(left >= right))
            },

            TokenType::Less => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Val::Bool(left < right))
            },

            TokenType::LessEqual => {
                let left = assert_num(&op, left)?;
                let right = assert_num(&op, right)?;

                Ok(Val::Bool(left <= right))
            },

            TokenType::BangEqual => Ok(Val::Bool(left != right)),

            TokenType::EqualEqual => Ok(Val::Bool(left == right)),

            _ => unreachable!()
        }
    }
}

fn is_truthy(value: &Val) -> bool {
    match value {
        Val::Nil => false,
        Val::Bool(b) => *b,
        _ => true
    }
}

fn assert_str(op: &Token, lit: Val) -> Result<Rc<String>> {
    if let Val::Str(str) = lit {
       Ok(str)
    } else {
        Err(Spanned { value: LoxError::TypeError("string"), span: op.span })
    }
}

fn assert_num(op: &Token, lit: Val) -> Result<f64> {
    if let Val::Num(num) = lit {
       Ok(num)
    } else {
        Err(Spanned { value: LoxError::TypeError("number"), span: op.span })
    }
}

fn assert_bool(op: &Token, lit: Val) -> Result<bool> {
    if let Val::Bool(boolean) = lit {
       Ok(boolean)
    } else {
        Err(Spanned { value: LoxError::TypeError("bool"), span: op.span })
    }
}

#[derive(Debug, Clone)]
pub enum LoxValue {
    Bool(bool),
    Num(f64),
    Str(Rc<String>),
    Nil,
    NativeFunction(Rc<dyn Call>),
    Function(Rc<LoxFunction>),
    Class(Rc<Class>),
    Instance(Instance),
}

impl PartialEq for LoxValue {
    fn eq(&self, other: &Self) -> bool {
        if self.is_nil() && other.is_nil() {
            return true;
        }

        if self.is_nil() {
            return false;
        }

        if let (Self::Num(left), Self::Num(right)) = (&self, &other) {
            return left == right;
        }

        if let (Self::Str(left), Self::Str(right)) = (&self, &other) {
            return left == right;
        }

        if let (Self::Bool(left), Self::Bool(right)) = (&self, &other) {
            return left == right;
        }

        if let (Self::Function(left), Self::Function(right)) = (&self, &other) {
            return Rc::ptr_eq(left, right);
        }

        if let (Self::NativeFunction(left), Self::NativeFunction(right)) = (&self, &other) {
            return Rc::ptr_eq(left, right);
        }

        if let (Self::Class(left), Self::Class(right)) = (&self, &other) {
            return Rc::ptr_eq(left, right);
        }

        false
    }
}

impl Eq for LoxValue {}

impl Hash for LoxValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state)
    }
}

impl LoxValue {
    pub fn is_bool(&self) -> bool {
        match self {
            Self::Bool(_) => true,
            _ => false,
        }
    }

    pub fn is_num(&self) -> bool {
        match self {
            Self::Num(_) => true,
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        match self {
            Self::Str(_) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Self::Nil => true,
            _ => false,
        }
    }
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Nil => write!(f, "nil"),
            LoxValue::Num(val) => write!(f, "{val}"),
            LoxValue::Bool(val) => write!(f, "{val}"),
            LoxValue::Str(val) => write!(f, "{val}"),
            LoxValue::Function(val) => write!(f, "{val}"),
            LoxValue::NativeFunction(val) => write!(f, "{val}"),
            LoxValue::Class(val) => write!(f, "{val}"),
            LoxValue::Instance(instance) => write!(f, "{}", instance),
        }
    }
}

impl From<Literal> for LoxValue {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Nil => Self::Nil,
            Literal::Num(val) => Self::Num(val),
            Literal::Bool(val) => Self::Bool(val),
            Literal::Str(val) => Self::Str(val),
        }
    }
}

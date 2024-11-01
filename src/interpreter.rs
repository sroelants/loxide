#![allow(dead_code)]
use crate::ast::Ast;
use crate::ast::LoxLiteral as Lit;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::environment::Env;
use crate::errors::BaseError;
use crate::errors::Stage;
use crate::tokens::Token;
use crate::tokens::TokenType;

type Result<T> = std::result::Result<T, BaseError>;

pub struct Interpreter {
    env: Env
}

impl Interpreter {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }

    pub fn interpret(&mut self, ast: Ast) -> Result<Lit> {
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
                self.eval_block(statements)?;
            }
        };

        Ok(Lit::Nil)
    }

    fn eval_block(&mut self, statements: &Vec<Stmt>) -> Result<Lit> {
        self.env.push_scope();

        for statement in statements.iter() {
            if let Err(err) = self.execute(statement) {
                self.env.pop_scope();
                return Err(err);
            }
        }

        self.env.pop_scope();
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

            Expr::Variable { name } => self.env.get(name),

            Expr::Assignment { name, value } => {
                let value = self.evaluate(value)?;
                self.env.assign(name, value.clone())?;
                Ok(value)
            }
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
                return Err(BaseError {
                    stage: Stage::Runtime,
                    span: token.span, //TODO Wouldn't I rather store the function identifier/expression?
                    msg: format!("expected {} arguments, but got {}", args.len(), fun.arity())
                })
            }

            Ok(fun.call(self, &evaluated_args))
        } else {
            Err(BaseError {
                stage: Stage::Runtime,
                span: token.span, //TODO Wouldn't I rather store the function identifier/expression?
                msg: format!("Expression is not callable")
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
                    Err(BaseError {
                        stage: Stage::Runtime,
                        span: op.span,
                        msg: format!("operands must be string or number")
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
        Err(BaseError { stage: Stage::Runtime, span: op.span, msg: format!("operand must be string") })
    }
}

fn assert_num(op: &Token, lit: Lit) -> Result<f64> {
    if let Lit::Num(num) = lit {
       Ok(num)
    } else {
        Err(BaseError { stage: Stage::Runtime, span: op.span, msg: format!("operand must be number") })
    }
}

fn assert_bool(op: &Token, lit: Lit) -> Result<bool> {
    if let Lit::Bool(boolean) = lit {
       Ok(boolean)
    } else {
       Err(BaseError { stage: Stage::Runtime, span: op.span, msg: format!("operand must be boolean") })
    }
}

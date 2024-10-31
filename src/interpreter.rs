#![allow(dead_code)]
use crate::ast::Ast;
use crate::ast::LoxLiteral as Lit;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::environment::Env;
use crate::tokens::Token;
use crate::tokens::TokenType;

pub struct RuntimeError {
    pub token: Token,
    pub msg: String,
}

type Result<T> = std::result::Result<T, RuntimeError>;

pub struct Interpreter {
    env: Env
}

impl Interpreter {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }

    pub fn interpret(&mut self, ast: Ast) -> Result<Lit> {
        for statement in ast {
            self.interpret_stmt(statement)?;
        }

        Ok(Lit::Nil)
    }

    fn interpret_stmt(&mut self, statement: Stmt) -> Result<Lit> {
        match statement {
            Stmt::Print { expr } => {
                let val = self.interpret_expr(expr)?;
                println!("{val}");
            }

            Stmt::Expression { expr } => {
               self.interpret_expr(expr)?;
            }

            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.interpret_expr(expr)?
                } else {
                    Lit::Nil
                };

                self.env.define(name, value);
            }

            Stmt::Block { statements } => {
                self.interpret_block(statements)?;
            }
        };

        Ok(Lit::Nil)
    }

    fn interpret_block(&mut self, statements: Vec<Stmt>) -> Result<Lit> {
        self.env.push_scope();

        for statement in statements {
            if let Err(err) = self.interpret_stmt(statement) {
                self.env.pop_scope();
                return Err(err);
            }
        }

        self.env.pop_scope();
        Ok(Lit::Nil)
    }

    fn interpret_expr(&mut self, expr: Expr) -> Result<Lit> {
        match expr {
            Expr::Literal { value } => Ok(value),

            Expr::Grouping { expr } => self.interpret_expr(*expr),

            Expr::Unary { op, right } => self.interpret_unary(op, *right),

            Expr::Binary { op, left, right } => self.interpret_binary(op, *left, *right),

            Expr::Variable { name } => self.env.get(name),

            Expr::Assignment { name, value } => {
                let value = self.interpret_expr(*value)?;
                self.env.assign(name, value.clone())?;
                Ok(value)
            }
        }
    }

    fn interpret_unary(&mut self, op: Token, right: Expr) -> Result<Lit> {
        let right = self.interpret_expr(right)?;

        match op.token_type {
            TokenType::Bang => Ok(Lit::Bool(!is_truthy(right))),

            TokenType::Minus => {
                let num = assert_num(&op, right)?;
                Ok(Lit::Num(-num))
            },

            _ => unreachable!(),
        }
    }

    fn interpret_binary(&mut self, op: Token, left: Expr, right: Expr) -> Result<Lit> {
        let left = self.interpret_expr(left)?;
        let right = self.interpret_expr(right)?;

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
                    Err(RuntimeError {
                        token: op,
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

fn is_truthy(value: Lit) -> bool {
    match value {
        Lit::Nil => false,
        Lit::Bool(b) => b,
        _ => true
    }
}

fn assert_str(op: &Token, lit: Lit) -> Result<String> {
    if let Lit::Str(str) = lit {
       Ok(str)
    } else {
        Err(RuntimeError { token: op.clone(), msg: format!("operand must be string") })
    }
}

fn assert_num(op: &Token, lit: Lit) -> Result<f64> {
    if let Lit::Num(num) = lit {
       Ok(num)
    } else {
        Err(RuntimeError { token: op.clone(), msg: format!("operand must be number") })
    }
}

fn assert_bool(op: &Token, lit: Lit) -> Result<bool> {
    if let Lit::Bool(boolean) = lit {
       Ok(boolean)
    } else {
       Err(RuntimeError { token: op.clone(), msg: format!("operand must be boolean") })
    }
}

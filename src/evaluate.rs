#![allow(dead_code)]
use crate::ast::Ast;
use crate::ast::LoxLiteral as Lit;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::tokens::Token;
use crate::tokens::TokenType;

pub struct RuntimeError {
    pub token: Token,
    pub msg: String,
}

type Result<T> = std::result::Result<T, RuntimeError>;

pub trait Interpret {
    fn interpret(self) -> Result<Lit>;
}

impl Interpret for Expr {
    fn interpret(self) -> Result<Lit> {
        match self {
            Expr::Literal { value } => Ok(value),
            Expr::Grouping { expr } => Ok(expr.interpret()?),
            Expr::Unary { op, right } => Ok(eval_unary(op, *right)?),
            Expr::Binary { op, left, right } => Ok(eval_binary(op, *left, *right)?),
        }
    }
}

fn eval_unary(op: Token, right: Expr) -> Result<Lit> {
    let right = right.interpret()?;

    match op.token_type {
        TokenType::Bang => Ok(Lit::Bool(!is_truthy(right))),

        TokenType::Minus => {
            let num = assert_num(&op, right)?;
            Ok(Lit::Num(-num))
        },

        _ => unreachable!(),
    }
}

fn eval_binary(op: Token, left: Expr, right: Expr) -> Result<Lit> {
    let left = left.interpret()?;
    let right = right.interpret()?;

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

impl Interpret for Stmt {
    fn interpret(self) -> Result<Lit> {
        match self {
            Stmt::Print { expr } => {
                let val = expr.interpret()?;
                println!("{val}");
            }

            Stmt::Expression { expr } => {
               expr.interpret()?;
            }
        };

        Ok(Lit::Nil)
    }
}

impl Interpret for Ast {
    fn interpret(self) -> Result<Lit> {
        for statement in self {
            statement.interpret()?;
        }

        Ok(Lit::Nil)
    }
}

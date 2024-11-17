use std::collections::HashMap;

use crate::sourcemap::Source;
use crate::span::Spanned;
use crate::errors::LoxError;
use crate::syntax::ast::{Ast, Expr, Stmt};
use crate::syntax::tokens::Token;

use super::Visitor;

pub struct Resolver<'a> {
    source: &'a Source<'a>,
    scopes: Vec<HashMap<String, bool>>,
    pub locals: HashMap<&'a Expr, usize>,

}

enum FunctionType {
    Function,
    Method,
}

type ResolverResult = Result<(), Spanned<LoxError>>;

impl<'a> Resolver<'a> {
    pub fn new(source: &'a Source<'a>) -> Self {
        Self {
            source,
            scopes: Vec::new(),
            locals: HashMap::new(),
        }
    }

    fn error(&self, spanned: Spanned<LoxError>) {
        eprintln!("{}", self.source.annotate(spanned));
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_many(&mut self, statements: &'a [Stmt]) -> ResolverResult {
        for statement in statements {
            self.visit(statement)?;
        }

        Ok(())
    }

    fn resolve_fun(
        &mut self,
        _fun_type: FunctionType,
        _name: &Token,
        params: &[Token],
        body: &'a [Stmt]
    ) -> ResolverResult {
        self.push_scope();

        for param in params {
            self.declare(param);
            self.define(param);
        }

        self.resolve_many(body)?;

        self.pop_scope();

        Ok(())
    }

    fn resolve_class(&mut self, name: &Token) -> ResolverResult {
        self.declare(name);
        self.define(name);
        Ok(())
    }

    pub fn resolve_local(&mut self, expr: &'a Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.locals.insert(expr, i);
                break;
            }
        }
    }
}

impl<'a> Visitor<&'a Stmt> for Resolver<'a> {
    type Output = Result<(), Spanned<LoxError>>;

    fn visit(&mut self, stmt: &'a Stmt) -> ResolverResult {
        match stmt {
            Stmt::Block { statements } => {
                self.push_scope();
                self.resolve_many(statements)?;
                self.pop_scope();
            },

            Stmt::Var { name, initializer } => {
                self.declare(&name);

                if let Some(initializer) = initializer {
                    self.visit(initializer)?;
                }

                self.define(&name);
            },

            Stmt::Fun { name, params, body } => {
                self.declare(name);
                self.define(name);

                self.resolve_fun(FunctionType::Function, name, params, body)?;
            },

            Stmt::Expression { expr } => {
                self.visit(expr)?;
            },

            Stmt::If { condition, then_branch, else_branch } => {
                self.visit(condition)?;
                self.visit(then_branch.as_ref())?;
                if let Some(else_branch) = else_branch {
                    self.visit(else_branch.as_ref())?;
                }
            },

            Stmt::Print { expr } => {
                self.visit(expr)?;
            },

            Stmt::Return { expr, .. } => {
                if let Some(expr) = expr {
                    self.visit(expr)?;
                }
            },

            Stmt::While { condition, body } => {
                self.visit(condition)?;
                self.visit(body.as_ref())?;
            },

            Stmt::Class { name, methods } => {
                self.resolve_class(name)?;

                self.push_scope();

                self.scopes.last_mut().unwrap().insert("this".to_owned(), true);

                for method in methods {
                    if let Stmt::Fun { name, params, body } = method {
                        self.resolve_fun(FunctionType::Method, name, params, body)?;
                    }
                }

                self.pop_scope();
            }
        }

        Ok(())
    }
}

impl<'a> Visitor<&'a Expr> for Resolver<'a> {
    type Output = Result<(), Spanned<LoxError>>;

    fn visit(&mut self, expr: &'a Expr) -> ResolverResult {
        match expr {
            Expr::Variable { name } => {
                if let Some(scope) = self.scopes.last() {
                    if scope.get(&name.lexeme).is_some_and(|v| !v) {
                        self.error(Spanned {
                            value: LoxError::RecursiveVarDecl,
                            span: name.span,
                        });
                    }
                }

                self.resolve_local(&expr, name);
            },

            Expr::Assignment { name, value } => {
                self.visit(value.as_ref())?;
                self.resolve_local(expr, name);
            },

            Expr::Binary { left, right, .. } => {
                self.visit(left.as_ref())?;
                self.visit(right.as_ref())?;
            },

            Expr::Call { callee, arguments, .. } => {
                self.visit(callee.as_ref())?;

                for arg in arguments {
                    self.visit(arg)?;
                }
            },

            Expr::Grouping { expr } => {
                self.visit(expr.as_ref())?;
            },

            Expr::Literal { .. } => {},

            Expr::Logical { left, right, .. } => {
                self.visit(left.as_ref())?;
                self.visit(right.as_ref())?;
            },

            Expr::Unary { right, .. } => {
                self.visit(right.as_ref())?;
            },

            Expr::Get { object, .. } => {
                self.visit(object.as_ref())?;
            },
            Expr::Set { value, object, .. } => {
                self.visit(value.as_ref())?;
                self.visit(object.as_ref())?;
            },

            Expr::This { keyword  } => {
                self.resolve_local(expr, keyword);
            }
        }

        Ok(())
    }
}

impl<'a> Visitor<&'a Ast> for Resolver<'a> {
    type Output = ResolverResult;

    fn visit(&mut self, ast: &'a Ast) -> ResolverResult {
        self.resolve_many(ast)?;

        Ok(())
    }
}

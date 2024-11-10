use std::collections::HashMap;

use crate::{ast::{Expr, Stmt}, errors::LoxError, span::Spanned, tokens::Token};

pub struct Resolver<'a> {
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<Spanned<LoxError>>,
    pub locals: HashMap<&'a Expr, usize>,

}

enum FunctionType {
    None,
    Function,
    Method,
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            errors: Vec::new(),
            locals: HashMap::new(),
        }
    }

    pub fn resolve_ast(&mut self, ast: &'a [Stmt]) {
        self.resolve_many(ast);
    }

    pub fn resolve_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Block { statements } => {
                self.push_scope();
                self.resolve_many(statements);
                self.pop_scope();
            },

            Stmt::Var { name, initializer } => {
                self.declare(&name);

                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer);
                }

                self.define(&name);
            },

            Stmt::Fun { name, params, body } => {
                self.declare(name);
                self.define(name);

                self.resolve_fun(FunctionType::Function, name, params, body);
            },

            Stmt::Expression { expr } => {
                self.resolve_expr(expr);
            },

            Stmt::If { condition, then_branch, else_branch } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch);
                }
            },

            Stmt::Print { expr } => {
                self.resolve_expr(expr);
            },

            Stmt::Return { expr, .. } => {
                if let Some(expr) = expr {
                    self.resolve_expr(expr);
                }
            },

            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(body);
            },

            Stmt::Class { name, methods } => {
                self.resolve_class(name);

                for method in methods {
                    if let Stmt::Fun { name, params, body } = method {
                        self.resolve_fun(FunctionType::Method, name, params, body);
                    }
                }
            }
        }
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

    fn resolve_many(&mut self, statements: &'a [Stmt]) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Variable { name } => {
                if let Some(scope) = self.scopes.last() {
                    if scope.get(&name.lexeme).is_some_and(|v| !v) {
                        self.errors.push( Spanned {
                            value: LoxError::RecursiveVarDecl,
                            span: name.span,
                        })
                    }
                }

                self.resolve_local(&expr, name);
            },

            Expr::Assignment { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
            },

            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            },

            Expr::Call { callee, arguments, .. } => {
                self.resolve_expr(callee);

                for arg in arguments {
                    self.resolve_expr(arg);
                }
            },

            Expr::Grouping { expr } => {
                self.resolve_expr(expr);
            },

            Expr::Literal { .. } => {},

            Expr::Logical { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            },

            Expr::Unary { right, .. } => {
                self.resolve_expr(right);
            },

            Expr::Get { object, .. } => {
                self.resolve_expr(object);
            },
            Expr::Set { value, object, .. } => {
                self.resolve_expr(value);
                self.resolve_expr(object);
            }
        }
    }

    fn resolve_fun(&mut self, _fun_type: FunctionType, _name: &Token, params: &[Token], body: &'a [Stmt]) {
        self.push_scope();

        for param in params {
            self.declare(param);
            self.define(param);
        }

        self.resolve_many(body);

        self.pop_scope();
    }

    fn resolve_class(&mut self, name: &Token) {
        self.declare(name);
        self.define(name);
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

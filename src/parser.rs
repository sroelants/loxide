//! The Parser will take a vector of tokens (or an iterator?)
//! and iterates over it to construct an `Expr` tree.
//! We should be able to use a lot of the same logic, i.e., peeking and advancing
//! the iterator.

use std::iter::Peekable;
use std::rc::Rc;
use std::vec::IntoIter;
use crate::ast::Ast;
use crate::ast::Literal;
use crate::ast::Stmt;
use crate::errors::LoxError;
use crate::span::Span;
use crate::span::Spanned;
use crate::tokens::Token;
use crate::tokens::TokenType;
use crate::ast::Expr;

type ParseResult<T> = Result<T, Spanned<LoxError>>;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    errors: Vec<Spanned<LoxError>>,
    span: Span,

}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
            errors: Vec::new(),
            span: Span::new(),
        }
    }

    pub fn finished(&mut self) -> bool {
        if let Some(next) = self.tokens.peek() {
            next.token_type == TokenType::Eof
        } else {
           true
        }
    }

    pub fn errors(&self) -> &[Spanned<LoxError>] {
        self.errors.as_slice()
    }

    fn error(&mut self, err: LoxError) -> Spanned<LoxError> {
        Spanned { value: err, span: self.span }
    }

    /// Checks whether the next token matches the provided type, without
    /// consuming the token.
    fn check(&mut self, token_type: TokenType) -> bool {
        self.tokens.peek().is_some_and(|t| t.token_type == token_type)
    }

    fn consume(&mut self) -> Option<Token> {
        if let Some(peeked) = self.tokens.peek() {
            self.span = peeked.span;
        }

        self.tokens.next()
    }

    #[allow(dead_code)]
    /// Consume and discard tokens until we get back to an unambiguous beginning
    /// of a new expression/statement.
    fn synchronize(&mut self) {
        use TokenType::*;

        while let Some(token) = self.consume() {
            if token.token_type == Semicolon {
                return;
            }

            let Some(next) = self.tokens.peek() else { continue };

            match next.token_type {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => {}
            };
        }
    }

    /// Check whether the next token matches the provided token type and, if so,
    /// consumes the matched token
    pub fn matches(&mut self, ttype: TokenType) -> Option<Token> {
        if self.check(ttype) {
            return self.consume()
        } else {
            None
        }
    }

    /// Check whether the next token matches any of the provided types, and
    /// consumes the matched token
    pub fn match_any(&mut self, types: &[TokenType]) -> Option<Token> {
        for ttype in types {
            if self.check(*ttype) {
                return self.consume();
            }
        }

        None
    }

    pub fn expect(&mut self, expected: TokenType, err: LoxError) -> ParseResult<Token> {
        let Some(next) = self.tokens.peek() else {
            return Err(Spanned { value: err, span: self.span })
        };

        if next.token_type != expected {
            return Err(Spanned { value: err, span: self.span })
        }

        Ok(self.consume().unwrap())
    }

    pub fn declaration(&mut self) -> ParseResult<Stmt> {
        if let Some(_) = self.matches(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    pub fn var_declaration(&mut self) -> ParseResult<Stmt> {
        use TokenType::*;
        let name = self.expect(Identifier, LoxError::ExpectedVarName)?;

        let initializer = if let Some(_) = self.matches(Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.expect(Semicolon, LoxError::ExpectedSemicolon)?;

        Ok(Stmt::Var { name, initializer })
    }

    pub fn statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::*;

        if let Some(keyword) = self.matches(Return) {
            self.return_statement(keyword)
        } else if let Some(_) = self.matches(Class) {
            self.class()
        } else if let Some(_) = self.matches(Fun) {
            self.function("function")
        } else if let Some(_) = self.matches(If) {
            self.if_statement()
        } else if let Some(_) = self.matches(While) {
            self.while_statement()
        } else if let Some(_) = self.matches(For) {
            self.for_statement()
        } else if let Some(_) = self.matches(Print) {
            self.print_statement()
        } else if let Some(_) = self.matches(LeftBrace) {
            Ok(Stmt::Block { statements: self.block()? })
        } else {
            self.expression_statement()
        }
    }

    pub fn class(&mut self) -> ParseResult<Stmt> {
        use TokenType::*;
        let name = self.expect(Identifier, LoxError::ExpectedClassName)?;
        self.expect(LeftBrace, LoxError::ExpectedLeftBrace("before class body"))?;

        let mut methods = Vec::new();

        while !self.check(RightBrace) && !self.finished() {
            methods.push(self.function("method")?);
        }

        self.expect(RightBrace, LoxError::ExpectedRightBrace("after class body"))?;

        Ok(Stmt::Class { name, methods })
    }

    pub fn return_statement(&mut self, keyword: Token) -> ParseResult<Stmt> {
        let expr = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.expect(TokenType::Semicolon, LoxError::ExpectedSemicolon)?;
        Ok(Stmt::Return { keyword, expr })
    }

    pub fn function(&mut self, _kind: &str) -> ParseResult<Stmt> {
        use TokenType::*;

        // Parse identifier
        let name = self.expect(Identifier, LoxError::ExpectedFunName)?;
        self.expect(LeftParen, LoxError::ExpectedLeftParen("after function name"))?;

        // Parse params
        let mut params = Vec::new();

        if !self.check(RightParen) {
            params.push(self.expect(Identifier, LoxError::ExpectedParamName(""))?);

            while let Some(_) = self.matches(Comma) {
                if params.len() >= 255 {
                    self.errors.push(Spanned {
                        value: LoxError::TooManyParams,
                        span: self.tokens.peek().unwrap().span,
                    })
                }

                params.push(
                    self.expect(Identifier, LoxError::ExpectedParamName(""))?
                );
            }
        }

        self.expect(RightParen, LoxError::ExpectedRightParen("after parameters"))?;

        // Parse body
        self.expect(LeftBrace, LoxError::ExpectedLeftBrace("before function body"))?;
        let body = self.block()?;

        Ok(Stmt::Fun { name, params, body })
    }

    pub fn if_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::*;

        self.expect(LeftParen, LoxError::ExpectedLeftParen("after 'if'"))?;
        let condition = self.expression()?;
        self.expect(RightParen, LoxError::ExpectedRightParen("after if condition"))?;

        let then_branch = Box::new(self.statement()?);

        let else_branch = if let Some(_) = self.matches(Else) {
            Some(Box::new(self.statement()?))
        } else {
           None
        };

        Ok(Stmt::If { condition, then_branch, else_branch })
    }

    pub fn while_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::*;
        self.expect(LeftParen, LoxError::ExpectedLeftParen("after 'while'"))?;
        let condition = self.expression()?;
        self.expect(RightParen, LoxError::ExpectedRightParen("after while condition"))?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    pub fn for_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::*;

        self.expect(LeftParen, LoxError::ExpectedLeftParen("after 'for'"))?;

        let initializer = if let Some(_) = self.matches(Semicolon) {
            None
        } else if let Some(_) = self.matches(Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.expect(Semicolon, LoxError::ExpectedSemicolon)?;

        let increment = if !self.check(RightParen) {
            Some(self.expression()?)
        } else {
           None
        };

        self.expect(RightParen, LoxError::ExpectedRightParen("after for clause"))?;

        let mut body = self.statement()?;

        // Rewrite into a while-loop based AST
        if let Some(increment) = increment {
            body = Stmt::Block { statements: vec![
                body,
                Stmt::Expression { expr: increment },
            ]};
        }

        let condition = condition
            .unwrap_or(Expr::Literal { value: Literal::Bool(true) });
        body = Stmt::While { condition, body: Box::new(body) };

        if let Some(initializer) = initializer {
            body = Stmt::Block { statements: vec![initializer, body] }
        }

        Ok(body)
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::*;
        let expr = self.expression()?;

        self.expect(Semicolon, LoxError::ExpectedSemicolon)?;

        Ok(Stmt::Print { expr })
    }

    fn block(&mut self) -> ParseResult<Vec<Stmt>> {
        use TokenType::*;
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.finished() {
            statements.push(self.declaration()?)
        }

        self.expect(RightBrace, LoxError::ExpectedRightBrace("after block"))?;
        Ok(statements)
    }


    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::*;
        let expr = self.expression()?;

        self.expect(Semicolon, LoxError::ExpectedSemicolon)?;

        Ok(Stmt::Expression { expr })
    }

    pub fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    pub fn assignment(&mut self) -> ParseResult<Expr> {
        use TokenType::*;
        let expr = self.or()?;

        if let Some(_) = self.matches(Equal) {
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assignment { name, value: Box::new(value) });
            } else if let Expr::Get { name, object } = expr {
                return Ok(Expr::Set { name, object, value: Box::new(value) });
            }

            return Err(self.error(LoxError::InvalidAssigTarget));
        }

        return Ok(expr);
    }

    pub fn or(&mut self) -> ParseResult<Expr> {
        use TokenType::*;
        let mut expr = self.and()?;

        while let Some(op) = self.matches(Or) {
            let right = self.and()?;
            expr = Expr::Logical { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }

    pub fn and(&mut self) -> ParseResult<Expr> {
        use TokenType::*;
        let mut expr = self.equality()?;

        while let Some(op) = self.matches(And) {
            let right = self.equality()?;
            expr = Expr::Logical { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }

    pub fn equality(&mut self) -> ParseResult<Expr> {
        use TokenType::*;
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_any(&[BangEqual, EqualEqual]) {
            let right = self.comparison()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }



    pub fn comparison(&mut self) -> ParseResult<Expr> {
        use TokenType::*;
        let mut expr = self.term()?;

        while let Some(op) = self.match_any(&[Greater, GreaterEqual, Less, LessEqual]) {
            let right = self.term()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }

    pub fn term(&mut self) -> ParseResult<Expr> {
        use TokenType::*;
        let mut expr = self.factor()?;

        while let Some(op) = self.match_any(&[Minus, Plus]) {
            let right = self.factor()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }

    pub fn factor(&mut self) -> ParseResult<Expr> {
        use TokenType::*;
        let mut expr = self.unary()?;

        while let Some(op) = self.match_any(&[Slash, Star]) {
            let right = self.unary()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }

    pub fn unary(&mut self) -> ParseResult<Expr> {
        use TokenType::*;

        if let Some(op) = self.match_any(&[Bang, Minus]) {
            let right = self.unary()?;
            Ok(Expr::Unary { op, right: Box::new(right) })
        } else {
            self.call()
        }
    }

    pub fn call(&mut self) -> ParseResult<Expr> {
        use TokenType::*;

        // Parse function expression
        // (remember: primary can also be a parenthesized expression)
        let mut expr = self.primary()?;

        // deal with chained function calls (curried functions)
        loop {
            if let Some(_) = self.matches(LeftParen) {
                expr = self.finish_call(expr)?;
            } if let Some(_) = self.matches(Dot) {
                let name = self.expect(Identifier, LoxError::ExpectedPropertyName("after ."))?;
                expr = Expr::Get { name, object: Box::new(expr) }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        use TokenType::*;
        let mut arguments = Vec::new();

        if !self.check(RightParen) {
            // match the first argument,
            arguments.push(self.expression()?);

            // match any following arguments, followed by a comma
            while let Some(_) = self.matches(Comma) {
                if arguments.len() >= 255 {
                    self.errors.push(Spanned {
                        value: LoxError::TooManyArgs,
                        span: self.tokens.peek().unwrap().span,
                    });
                }

                arguments.push(self.expression()?);
            }
        }

        let paren = self.expect(RightParen, LoxError::ExpectedRightBrace("after arguments"))?;

        Ok(Expr::Call { callee: Box::new(callee), paren, arguments })
    }

    pub fn primary(&mut self) -> ParseResult<Expr> {
        use TokenType::*;

        if let Some(keyword) = self.matches(This) {
            return Ok(Expr::This { keyword });
        }

        if let Some(_) = self.matches(False) {
            return Ok(Expr::Literal { value: Literal::Bool(false) });
        }

        if let Some(_) = self.matches(True) {
            return Ok(Expr::Literal { value: Literal::Bool(true) });
        }

        if let Some(_) = self.matches(Nil) {
            return Ok(Expr::Literal { value: Literal::Nil });
        }

        if let Some(token) = self.matches(TokenType::String) {
            let value = token.lexeme;
            let len = value.len();
            let trimmed = &value[1..len-1];

            return Ok(Expr::Literal { value: Literal::Str(Rc::new(trimmed.to_owned())) });
        }

        if let Some(token) = self.matches(Number) {
            // TODO: In theory this could fail? Can it though, if it got
            // tokenized correctly?
            let value: f64 = token.lexeme.parse().unwrap();
            return Ok(Expr::Literal { value: Literal::Num(value) });
        }

        if let Some(name) = self.matches(Identifier) {
           return Ok(Expr::Variable { name });
        }

        if let Some(_) = self.matches(LeftParen) {
            let expr = self.expression()?;
            self.expect(RightParen, LoxError::ExpectedRightParen(""))?;

            return Ok(Expr::Grouping { expr: Box::new(expr) });
        }

        Err(self.error(LoxError::ExpectedExpression))
    }

    pub fn parse(&mut self) -> Result<Ast, Vec<Spanned<LoxError>>> {
        let mut statements = Vec::new();

        while !self.finished() {
            match self.declaration() {
                Ok(statement) => {
                    statements.push(statement)
                },
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                }
            }
        }

        if self.errors.len() == 0 {
            Ok(statements)
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }
}

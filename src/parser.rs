//! The Parser will take a vector of tokens (or an iterator?)
//! and iterates over it to construct an `Expr` tree.
//! We should be able to use a lot of the same logic, i.e., peeking and advancing
//! the iterator.

use std::iter::Peekable;
use std::vec::IntoIter;
use crate::span::Span;
use crate::tokens::Token;
use crate::tokens::TokenType;
use crate::ast::Expr;
use crate::ast::LoxLiteral;

type ParseResult = Result<Expr, ParseError>;

pub struct ParseError {
    pub span: Span,
    pub msg: String,
}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    errors: Vec<ParseError>,
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

    pub fn errors(&self) -> &[ParseError] {
        self.errors.as_slice()
    }

    fn error(&mut self, msg: String) -> ParseError {
        ParseError { span: self.span, msg }
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

    pub fn expect(&mut self, expected: TokenType) -> bool {
        let Some(next) = self.tokens.peek() else {
            return false;
        };

        if next.token_type != expected {
            return false;
        }

        self.consume();
        true
    }

    pub fn expression(&mut self) -> ParseResult {
        self.equality()
    }

    pub fn equality(&mut self) -> ParseResult {
        use TokenType::*;
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_any(&[BangEqual, EqualEqual]) {
            let right = self.comparison()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }

    pub fn comparison(&mut self) -> ParseResult {
        use TokenType::*;
        let mut expr = self.term()?;

        while let Some(op) = self.match_any(&[Greater, GreaterEqual, Less, LessEqual]) {
            let right = self.term()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }

    pub fn term(&mut self) -> ParseResult {
        use TokenType::*;
        let mut expr = self.factor()?;

        while let Some(op) = self.match_any(&[Minus, Plus]) {
            let right = self.factor()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }

    pub fn factor(&mut self) -> ParseResult {
        use TokenType::*;
        let mut expr = self.unary()?;

        while let Some(op) = self.match_any(&[Slash, Star]) {
            let right = self.unary()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }

        Ok(expr)
    }

    pub fn unary(&mut self) -> ParseResult {
        use TokenType::*;

        if let Some(op) = self.match_any(&[Bang, Minus]) {
            let right = self.unary()?;
            Ok(Expr::Unary { op, right: Box::new(right) })
        } else {
            self.primary()
        }
    }

    pub fn primary(&mut self) -> ParseResult {
        use TokenType::*;

        if let Some(_) = self.matches(False) {
            return Ok(Expr::Literal { value: LoxLiteral::Bool(false) });
        }

        if let Some(_) = self.matches(True) {
            return Ok(Expr::Literal { value: LoxLiteral::Bool(true) });
        }

        if let Some(_) = self.matches(Nil) {
            return Ok(Expr::Literal { value: LoxLiteral::Nil });
        }

        if let Some(token) = self.matches(TokenType::String) {
            let value = token.lexeme;
            let len = value.len();
            let trimmed = &value[1..len-1];

            return Ok(Expr::Literal { value: LoxLiteral::Str(trimmed.to_owned()) });
        }

        if let Some(token) = self.matches(Number) {
            // TODO: In theory this could fail? Can it though, if it got
            // tokenized correctly?
            let value: f64 = token.lexeme.parse().unwrap();
            return Ok(Expr::Literal { value: LoxLiteral::Num(value) });
        }

        if let Some(_) = self.matches(LeftParen) {
            let expr = self.expression()?;

            if self.expect(RightParen) {
               return Ok(Expr::Grouping { expr: Box::new(expr) });
            } else {
                return Err(self.error(format!("Expected ')'")));
            }
        }

        Err(self.error(format!("expected expression")))
    }

    pub fn parse(&mut self) -> Expr {
        match self.expression() {
            Ok(expr) => return expr,
            Err(err) => {
                self.errors.push(err);
                Expr::Literal { value: LoxLiteral::Nil }
            }
        }
    }
}

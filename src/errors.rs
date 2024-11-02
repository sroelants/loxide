use std::fmt::Display;
use crate::ast::Expr;
use crate::colors::{RED, NORMAL};

use crate::sourcemap::SourceMap;
use crate::span::{Annotated, Span};

#[derive(Clone)]
pub enum LoxError {
    FileNotFound(&'static str),
    FailedToReadInput,

    // Lexer errors
    UnexpectedToken,
    UnterminatedString,

    // Parser errors
    TooManyParams,
    TooManyArgs,
    ExpectedIdent,
    ExpectedSemicolon,
    ExpectedFunName,
    ExpectedLeftBrace(&'static str),
    ExpectedRightBrace(&'static str),
    ExpectedLeftParen(&'static str),
    ExpectedRightParen(&'static str),
    ExpectedParamName(&'static str),
    InvalidAssigTarget,
    ExpectedVarName,
    ExpectedExpression,

    // Runtime errors
    ArityMismatch(usize, usize),
    NotCallable,
    TypeError(&'static str),
    MultiTypeError(&'static str),

    // Not actual errors
    Return(Expr),
}

impl Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxError::FileNotFound(file) => write!(f, "File not found: {file}"),
            LoxError::FailedToReadInput => write!(f, "Failed to read from stdin"),
            LoxError::UnexpectedToken => write!(f, "Unexpected input"),
            LoxError::UnterminatedString => write!(f, "Unterminated string"),

            LoxError::TooManyParams => write!(f, "Maximum number of parameters allowed is 255"),
            LoxError::TooManyArgs => write!(f, "Maximum number of arguments allowed is 255"),
            LoxError::ExpectedIdent => write!(f, "Expected identifier"),
            LoxError::ExpectedSemicolon => write!(f, "Expected ';' after statement"),
            LoxError::ExpectedFunName => write!(f, "Expected function name"),
            LoxError::ExpectedLeftBrace(ctx) => write!(f, "Expected '{{' {ctx}"),
            LoxError::ExpectedRightBrace(ctx) => write!(f, "Expected '}}' {ctx}"),
            LoxError::ExpectedLeftParen(ctx) => write!(f, "Expected '(' {ctx}"),
            LoxError::ExpectedRightParen(ctx) => write!(f, "Expected ')' {ctx}"),
            LoxError::ExpectedParamName(ctx) => write!(f, "Expected parameter name {ctx}"),
            LoxError::InvalidAssigTarget => write!(f, "Invalid assignment target"),
            LoxError::ExpectedVarName => write!(f, "Expected variable name"),
            LoxError::ExpectedExpression => write!(f, "Expected expression"),

            LoxError::ArityMismatch(expected, found) => write!(f, "Expected {expected} arguments, but found {found}"),
            LoxError::NotCallable => write!(f, "Expression is not callable"),
            LoxError::TypeError(ctx) => write!(f, "Operand must be {ctx}"),
            LoxError::MultiTypeError(ctx) => write!(f, "Operands must both be {ctx}"),
            LoxError::Return(_) => unreachable!()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Stage {
    Lexer,
    Parser,
    Runtime,
}

impl Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lexer => write!(f, "Lexer"),
            Self::Parser => write!(f, "Parser"),
            Self::Runtime => write!(f, "Runtime"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BaseError {
    pub stage: Stage,
    pub span: Span,
    pub msg: String,
}

pub struct RichError<'a> {
    stage: Stage,
    span: Span,
    msg: String,
    source: &'a str,
    line: usize,
    col: usize,
}

impl<'a> RichError<'a> {
    pub fn annotate(err: BaseError, sourcemap: &'a SourceMap<'a>) -> Self {
        let (line, col, source) = sourcemap.map_span(err.span);

        RichError {
            stage: err.stage,
            span: err.span,
            msg: err.msg,
            source,
            line,
            col,
        }
    }
}

impl<'a> Display for RichError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let marker_offset = self.col;
        let marker_len = self.span.len;
        writeln!(f, "{RED}ERR{NORMAL} ({}): {}:{} {}", self.stage, self.line, self.col, self.msg)?;
        writeln!(f, "    {}", self.source)?;
        writeln!(f, "    {RED}{: <marker_offset$}{:^>marker_len$}{NORMAL}","", "")
    }
}

impl<'a> Display for Annotated<'a, LoxError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let marker_offset = self.col;
        let marker_len = self.span.len;
        writeln!(f, "{RED}ERR{NORMAL} {}:{} {}", self.line, self.col, self.value)?;
        writeln!(f, "    {}", self.source)?;
        writeln!(f, "    {RED}{: <marker_offset$}{:^>marker_len$}{NORMAL}","", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_err() {
        let err = RichError {
            stage: Stage::Lexer,
            span: Span { offset: 0, len: 10 },
            msg: format!("something went wrong!"),
            line: 10,
            col: 5,
            source: "This is the offending line of source code that we're supposed to print",
        };

        println!("{err}");
        panic!()
    }
}

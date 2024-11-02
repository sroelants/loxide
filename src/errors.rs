use std::fmt::Display;
use crate::ast::LoxLiteral;
use crate::colors::{RED, NORMAL};

use crate::span::Annotated;

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
    UndeclaredVar(String),

    // Not actual errors
    Return(LoxLiteral),
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
            LoxError::UndeclaredVar(name) => write!(f, "Undeclared variable '{name}'"),

            // Not an actual error, should never make it to the error reporting
            // stage
            LoxError::Return(_) => unreachable!()
        }
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

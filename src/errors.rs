use std::fmt::Display;
use crate::colors::{RED, NORMAL};

use crate::sourcemap::SourceMap;
use crate::span::Span;

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

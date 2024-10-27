use crate::colors::{NORMAL, RED};
use std::fmt::Display;

#[derive(Debug)]
pub struct ScanError {
    line: usize,
    col: usize,
    ch: char,
}

impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{RED}ERR{NORMAL}] {line}:{col} Unexpected character '{ch}'", line = self.line, col = self.col, ch = self.ch)
    }
}

impl std::error::Error for ScanError {}

type ScanResult = Result<(), ScanError>;

#[derive(Debug, Copy, Clone)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One/two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Copy, Clone)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub line: usize,
    pub col: usize,
}

impl<'a> Token<'a> {
    pub fn new(token_type: TokenType, lexeme: &'a str, line: usize, col: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            col,
        }
    }
}

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line: usize,
    col: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokens(&self) -> &[Token] {
        self.tokens.as_slice()
    }

    pub fn completed(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_tokens(&mut self) {
        while !self.completed() {
            self.start = self.current;
            if let Err(err) = self.scan_token() {
                eprintln!("{err}");
            };
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "", self.line, self.col));
    }

    fn scan_token(&mut self) -> ScanResult {
        use TokenType::*;

        let ch = self.advance();

        let token_type = match ch {
            '(' => LeftParen,
            ')' => RightParen,
            '{' => LeftBrace,
            '}' => RightBrace,
            ',' => Comma,
            '.' => Dot,
            '-' => Minus,
            '+' => Plus,
            ';' => Semicolon,
            '*' => Star,

            // TODO: Keep matching until we get a maximal set of unrecognized characters,
            // so we can report them all in one go
            _ => return Err(ScanError { line: self.line, col: self.col, ch }),
        };

        let substr = &self.source.as_bytes()[self.start..self.current];
        let substr = std::str::from_utf8(substr).unwrap();

        // NOTE: I have to to the pushing inside this method to keep the borrow
        // checker happy. (It can't see the non-overlapping partial borrows if
        // we borrow from self.source and self.tokens at the same time.)
        let token = Token::new(token_type, substr, self.line, self.col);
        self.tokens.push(token);

        Ok(())
    }

    fn advance(&mut self) -> char {
        let ch = self.source.as_bytes()[self.current].into();
        self.current += 1;
        ch
    }
}

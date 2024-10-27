use crate::colors::{NORMAL, RED};
use std::fmt::Display;

#[derive(Debug)]
pub struct UnexpectedCharError {
    line: usize,
    col: usize,
    ch: char,
}

impl Display for UnexpectedCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{RED}ERR{NORMAL}] {line}:{col} Unexpected character '{ch}'", line = self.line, col = self.col, ch = self.ch)
    }
}

impl std::error::Error for UnexpectedCharError {}

#[derive(Debug)]
pub struct UnterminatedStringError {
    line: usize,
    col: usize,
}

impl Display for UnterminatedStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{RED}ERR{NORMAL}] {line}:{col} Unterminated string.", line = self.line, col = self.col)
    }
}

impl std::error::Error for UnterminatedStringError {}

type ScanResult = Result<(), UnexpectedCharError>;

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
            '!' => if self.matches('=') { BangEqual } else { Bang },
            '=' => if self.matches('=') { EqualEqual } else { Equal },
            '<' => if self.matches('=') { LessEqual } else { Less },
            '>' => if self.matches('=') { GreaterEqual } else { Greater },
            '/' => if self.matches('/') {
                while self.peek() != '\n' && !self.completed() {
                    self.advance();
                }
                return Ok(());
            } else {
                Slash
            },

            // Update column, but otherwise ignore whitespace
            ' ' | '\r' | '\t' => {
                self.col += 1;
                return Ok(());
            },

            // Update counters, but otherwise ignore newlines
            '\n' => {
                self.line += 1;
                self.col = 1;
                return Ok(());
            }

            '"' => {
                self.scan_string();
                TokenType::String
            },

            _ if ch.is_ascii_digit() => {
                self.scan_number();
                Number
            },

            // TODO: Keep matching until we get a maximal set of unrecognized characters,
            // so we can report them all in one go.
            _ => return Err(UnexpectedCharError { line: self.line, col: self.col, ch }),
        };

        let substr = &self.source.as_bytes()[self.start..self.current];
        let substr = std::str::from_utf8(substr).unwrap();

        // NOTE: I have to to the pushing inside this method to keep the borrow
        // checker happy. (It can't see the non-overlapping partial borrows if
        // we borrow from self.source and self.tokens at the same time.)
        let token = Token::new(token_type, substr, self.line, self.col);
        self.tokens.push(token);

        self.col += substr.len();

        Ok(())
    }

    fn advance(&mut self) -> char {
        let ch = self.char_at(self.current);
        self.current += 1;
        ch
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.completed() { return false; }
        if self.char_at(self.current) != expected { return false; }
        self.current += 1;

        true
    }

    fn char_at(&self, idx: usize) -> char {
        self.source.as_bytes()[idx].into()
    }

    fn peek(&self) -> char {
        if self.completed() { '\0' } else { self.char_at(self.current) }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() { return '\0'; }
        self.char_at(self.current + 1)
    }

    fn scan_string(&mut self) {
        while self.peek() != '"' && !self.completed() {
            if self.peek() == '\n' {
                self.line += 1;
                self.col = 1;
            }

            self.advance();
        }

        if self.completed() {
            // TODO: Handle the error correctly?

        }

        // Skip the closing '"'
        self.advance();
    }

    fn scan_number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the '.'
            self.advance();
        }

        while self.peek().is_ascii_digit() {
            self.advance();
        }
    }
}

use std::fmt::Display;
use std::iter::Peekable;
use std::str::Chars;

use crate::sourcemap::Source;
use crate::span::Span;
use crate::span::Spanned;
use super::tokens::Token;
use super::tokens::TokenType;

pub struct Scanner<'a> {
    source: &'a Source<'a>,
    finished: bool,
    chars: Peekable<Chars<'a>>,
    span: Span,
    had_error: bool
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a Source<'a>) -> Self {
        Self {
            source,
            finished: false,
            chars: source.source.chars().peekable(),
            span: Span::default(),
            had_error: false,
        }
    }

    /// Push a new LexError to the internal list of encountered errors
    fn error(&mut self, err: LexError) {
        let spanned = Spanned { value: err, span: self.span };
        eprintln!("{}", self.source.annotate(spanned));
        self.had_error = true;
    }

    /// Peek two characters ahead without advancing the internal iterator.
    fn peek_next(&self) -> Option<char> {
        self.chars.clone().skip(1).next()
    }

    /// Consume a character and update the internal span.
    fn consume_char(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        self.span.grow(ch.len_utf8());
        Some(ch)
    }

    /// Consume characters for as long as the given predicate holds.
    /// Does not consume the first character that returns false!
    fn consume_while<P>(&mut self, pred: P)
    where
        P: Fn(char) -> bool,
    {
        while self.chars.peek().is_some_and(|&ch| pred(ch)) {
            self.consume_char();
        }
    }

    /// Consume and return the next character only if it matches the
    /// provided char
    fn consume_if_eq(&mut self, to_match: char) -> Option<char> {
        if *self.chars.peek()? == to_match {
            self.consume_char()
        } else {
            None
        }
    }

    fn branch(&mut self, to_match: char, success: TokenType, failure: TokenType) -> TokenType {
        if self.consume_if_eq(to_match).is_some() {
            success
        } else {
            failure
        }
    }

    fn comment(&mut self) -> bool {
        if self.consume_if_eq('/').is_some() {
            self.consume_while(|ch| ch != '\n');
            true
        } else {
            false
        }
    }

    fn string(&mut self) -> bool {
        self.consume_while(|ch| ch != '"');

        // Check whether it's a correctly terminated string
        if self.consume_char() == Some('"') {
            true
        } else {
            self.error(LexError::UnterminatedString);
            false
        }
    }

    fn number(&mut self) {
        self.consume_while(|ch| ch.is_ascii_digit());

        if self.chars.peek().is_some_and(|&ch| ch == '.')
            && self.peek_next().is_some_and(|ch| ch.is_ascii_digit())
        {
            // Consume the '.'`
            self.consume_char();

            // Consume the rest of the number
            self.consume_while(|ch| ch.is_ascii_digit());
        }
    }

    fn identifier(&mut self) {
        self.consume_while(|ch| ch.is_ascii_alphanumeric() || ch == '_');
    }
}

// TODO: Maybe chain this somehow with a `std::iter::once(EOF)` after the fact.
impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenType::*;

        if self.finished {
            return None;
        }

        // Keep trying until we match a token, or return if
        // the characters iterator is exhausted.
        loop {
            // Start a new span
            self.span = Span::after(self.span);

            let Some(ch) = self.consume_char() else {
                self.finished = true;
                return Some(Token {
                    token_type: Eof,
                    span: self.span,
                    lexeme: "".to_owned(),
                });
            };

            let token_type = match ch {
                // Single character tokens
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

                // Two character tokens
                '!' => self.branch('=', BangEqual, Bang),
                '=' => self.branch('=', EqualEqual, Equal),
                '<' => self.branch('=', LessEqual, Less),
                '>' => self.branch('=', GreaterEqual, Greater),

                // Whitespace
                ' ' | '\n' | '\r' | '\t' => continue,

                // Comments
                '/' => {
                    // If it's a valid comment, match a new token
                    if self.comment() { continue; }
                    Slash
                }

                // Strings
                '"' => {
                    // If it's an illegal string, continue (and exit afterwards)
                    if !self.string() { continue; }
                    TokenType::String
                }

                // Numbers
                _ if ch.is_ascii_digit() => {
                    self.number();
                    Number
                }

                // Identifiers
                _ if ch.is_ascii_alphabetic() => {
                    self.identifier();
                    let ident = &self.source.source[self.span.range()];
                    ident_type(ident)
                }

                _ => {
                    self.error(LexError::UnexpectedToken);
                    continue;
                }
            };

            return Some(Token {
                token_type,
                span: self.span,
                lexeme: self.source.source[self.span.range()].to_owned(),
            });
        }
    }
}

// TODO: Pull in something like lazy_static! and make this a static hashmap
// (or phf and do it at compile-time)
fn ident_type(s: &str) -> TokenType {
    use TokenType::*;

    match s {
        "and" => And,
        "class" => Class,
        "else" => Else,
        "false" => False,
        "for" => For,
        "fun" => Fun,
        "if" => If,
        "nil" => Nil,
        "or" => Or,
        "print" => Print,
        "return" => Return,
        "super" => Super,
        "this" => This,
        "true" => True,
        "var" => Var,
        "while" => While,
        _ => Identifier,
    }
}

#[derive(Clone)]
pub enum LexError {
    UnexpectedToken,
    UnterminatedString,
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnexpectedToken => write!(f, "Unexpected input"),
            LexError::UnterminatedString => write!(f, "Unterminated string"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::tokens::Token;

    use super::*;

    #[test]
    fn minimal() {
        use TokenType::*;
        let source = Source::new(".");
        let scanner = Scanner::new(&source);

        assert_eq!(
            scanner.collect::<Vec<_>>(),
            vec![
                Token {
                    token_type: Dot,
                    span: Span { offset: 0, len: 1 },
                    lexeme: ".".to_owned()
                },
                Token {
                    token_type: Eof,
                    span: Span { offset: 1, len: 0 },
                    lexeme: "".to_owned()
                },
            ]
        );
    }

    #[test]
    fn single_tokens() {
        use TokenType::*;
        let source = Source::new("((.))");
        let scanner = Scanner::new(&source);
        assert_eq!(
            scanner.collect::<Vec<_>>(),
            vec![
                Token {
                    token_type: LeftParen,
                    span: Span { offset: 0, len: 1 },
                    lexeme: "(".to_owned()
                },
                Token {
                    token_type: LeftParen,
                    span: Span { offset: 1, len: 1 },
                    lexeme: "(".to_owned()
                },
                Token {
                    token_type: Dot,
                    span: Span { offset: 2, len: 1 },
                    lexeme: ".".to_owned()
                },
                Token {
                    token_type: RightParen,
                    span: Span { offset: 3, len: 1 },
                    lexeme: ")".to_owned()
                },
                Token {
                    token_type: RightParen,
                    span: Span { offset: 4, len: 1 },
                    lexeme: ")".to_owned()
                },
                Token {
                    token_type: Eof,
                    span: Span { offset: 5, len: 0 },
                    lexeme: "".to_owned()
                },
            ]
        );
    }

    #[test]
    fn two_char_tokens() {
        use TokenType::*;
        let source = Source::new("!=!");
        let scanner = Scanner::new(&source);
        assert_eq!(
            scanner.collect::<Vec<_>>(),
            vec![
                Token {
                    token_type: BangEqual,
                    span: Span { offset: 0, len: 2 },
                    lexeme: "!=".to_owned()
                },
                Token {
                    token_type: Bang,
                    span: Span { offset: 2, len: 1 },
                    lexeme: "!".to_owned()
                },
                Token {
                    token_type: Eof,
                    span: Span { offset: 3, len: 0 },
                    lexeme: "".to_owned()
                },
            ]
        );
    }

    #[test]
    fn comments() {
        use TokenType::*;
        let source = Source::new("() // lolol");
        let scanner = Scanner::new(&source);

        assert_eq!(
            scanner.collect::<Vec<_>>(),
            vec![
                Token {
                    token_type: LeftParen,
                    span: Span { offset: 0, len: 1 },
                    lexeme: "(".to_owned()
                },
                Token {
                    token_type: RightParen,
                    span: Span { offset: 1, len: 1 },
                    lexeme: ")".to_owned()
                },
                Token {
                    token_type: Eof,
                    span: Span { offset: 11, len: 0 },
                    lexeme: "".to_owned()
                },
            ]
        );
    }

    #[test]
    fn strings() {
        let source = Source::new(r#""Hello there!""#);
        let scanner = Scanner::new(&source);
        assert_eq!(
            scanner.collect::<Vec<_>>(),
            vec![
                Token {
                    token_type: TokenType::String,
                    span: Span { offset: 0, len: 14 },
                    lexeme: r#""Hello there!""#.to_owned()
                },
                Token {
                    token_type: TokenType::Eof,
                    span: Span { offset: 14, len: 0 },
                    lexeme: "".to_owned()
                },
            ]
        );
    }

    #[test]
    fn unterminated_strings() {
        let source = Source::new(r#""Hello there!"#);
        let mut scanner = Scanner::new(&source);

        // Consume the tokens
        for _ in scanner.by_ref() {}

        assert!(scanner.had_error);
    }

    #[test]
    fn numbers() {
        let source = Source::new("123, 123.0, 123.");
        let scanner = Scanner::new(&source);

        assert_eq!(
            scanner.collect::<Vec<_>>(),
            vec![
                Token {
                    token_type: TokenType::Number,
                    span: Span { offset: 0, len: 3 },
                    lexeme: "123".to_owned(),
                },
                Token {
                    token_type: TokenType::Comma,
                    span: Span { offset: 3, len: 1 },
                    lexeme: ",".to_owned(),
                },
                Token {
                    token_type: TokenType::Number,
                    span: Span { offset: 5, len: 5 },
                    lexeme: "123.0".to_owned(),
                },
                Token {
                    token_type: TokenType::Comma,
                    span: Span { offset: 10, len: 1 },
                    lexeme: ",".to_owned(),
                },
                Token {
                    token_type: TokenType::Number,
                    span: Span { offset: 12, len: 3 },
                    lexeme: "123".to_owned(),
                },
                Token {
                    token_type: TokenType::Dot,
                    span: Span { offset: 15, len: 1 },
                    lexeme: ".".to_owned(),
                },
                Token {
                    token_type: TokenType::Eof,
                    span: Span { offset: 16, len: 0 },
                    lexeme: "".to_owned(),
                },
            ]
        );
    }
}

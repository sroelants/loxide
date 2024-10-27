use std::env;
use std::fmt::Display;
use std::io::Write;
use std::path::PathBuf;

use scanner::{ScanError, Scanner, Token};
use colors::{NORMAL, RED};

mod scanner;
pub mod colors;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut interpreter = Loxide::new();

    if args.len() > 2 {
        println!("Usage: loxide [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        interpreter.run_file(&args[1]);
    } else {
        interpreter.run_prompt();
    }
}

struct Loxide {
    found_error: bool,
}

impl Loxide {
    pub fn new() -> Self {
        Self { found_error: false }
    }

    pub fn run_file(&mut self, file: &str) {
        let Ok(input) = std::fs::read_to_string(PathBuf::from(file)) else {
            eprintln!("[{RED}ERR{NORMAL}]: File not found: {file}");
            return;
        };

        if let Err(err) = self.run(&input) {
            eprintln!("{err}");
            self.found_error = true;
            std::process::exit(65);
        };
    }

    pub fn run_prompt(&mut self) {
        print_prompt();

        for line in std::io::stdin().lines() {
            let Ok(line) = line else {
                eprintln!("[{RED}ERR{NORMAL}] Failed to read input");
                print_prompt();
                continue;
            };


            if let Err(err) = self.run(&line) {
                eprintln!("{err}");
            };

            print_prompt();
        }
    }

    pub fn run(&mut self, input: &str) -> Result<(), LoxideError> {
        let mut scanner = Scanner::new(input);
        scanner.scan_tokens();

        for token in scanner.tokens() {
            println!("{token:?}");
        }

        Ok(())
    }
}

fn print_prompt() {
    print!("> ");
    std::io::stdout().flush().unwrap();
}

pub struct FileNotFoundError<'a> {
    path:  &'a str,
}

pub struct ReadFailedError;

pub struct ParseError<'a> {
    token: Token<'a>,
}

pub enum LoxideError<'a> {
    FileNotFoundError(FileNotFoundError<'a>),
    ReadFailedError(ReadFailedError),
    ScanError(ScanError),
    ParseError(ParseError<'a>),
}

impl<'a> Display for LoxideError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxideError::FileNotFoundError(err) =>
                write!(f, "[{RED}ERR{NORMAL}] File not found: {path}", path = err.path),
            LoxideError::ReadFailedError(_) =>
                write!(f, "[{RED}ERR{NORMAL}] Failed to read input"),
            LoxideError::ScanError(err) =>
                write!(f, "{err}"),
            LoxideError::ParseError(err) =>
                write!(f, "[{RED}ERR{NORMAL}] {line}:{col} Unexpected token {tok}", line = err.token.line, col = err.token.col, tok = err.token.lexeme),
            _ => Ok(())
        }
    }
}

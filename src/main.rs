use std::{env, error::Error};
use std::fmt::Display;
use std::io::Write;
use std::path::PathBuf;

use evaluate::EvalExpr;
use parser::Parser;
use tokenizer::Scanner;
use colors::{NORMAL, RED};
use tokens::Token;

pub mod tokenizer;
pub mod parser;
pub mod colors;
pub mod ast;
pub mod pretty_print;
pub mod tokens;
pub mod span;
pub mod evaluate;

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
    static_error: bool,
    runtime_error: bool,
}

impl Loxide {
    pub fn new() -> Self {
        Self {
            static_error: false,
            runtime_error: false,
        }
    }

    pub fn run_file(&mut self, file: &str) {
        let Ok(input) = std::fs::read_to_string(PathBuf::from(file)) else {
            eprintln!("[{RED}ERR{NORMAL}]: File not found: {file}");
            return;
        };

        let _ = self.run(&input);
    }

    pub fn run_prompt(&mut self) {
        print_prompt();

        for line in std::io::stdin().lines() {
            self.static_error = false;
            self.runtime_error = false;

            let Ok(line) = line else {
                eprintln!("[{RED}ERR{NORMAL}] Failed to read input");
                print_prompt();
                continue;
            };


            let _ = self.run(&line);

            print_prompt();
        }
    }

    pub fn run(&mut self, input: &str) {
        let mut scanner = Scanner::new(input);
        let tokens: Vec<Token> = scanner.by_ref().collect();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        // Move this up, somewhere else?
        for error in scanner.errors() {
            self.static_error = true;
            eprintln!("[{RED}ERR{NORMAL}] Lexer error: {}", error.msg);
        }

        for error in parser.errors() {
            self.static_error = true;
            eprintln!("[{RED}ERR{NORMAL}] Parse error: {}", error.msg);
        }

        // Evaluate
        match ast.eval() {
            Ok(lit) => println!("{lit}"),
            Err(error) => {
                self.runtime_error = true;
                eprintln!("[{RED}ERR{NORMAL}] Runtime error: {}", error.msg);
            }
        }
    }
}

fn print_prompt() {
    print!("> ");
    std::io::stdout().flush().unwrap();
}

#[derive(Debug)]
pub struct FileNotFoundError<'a> {
    path:  &'a str,
}

impl<'a> Display for FileNotFoundError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{RED}ERR{NORMAL}] File not found: {path}", path = self.path)
    }
}

impl<'a> Error for FileNotFoundError<'a> {}

#[derive(Debug)]
pub struct ReadFailedError;

impl Display for ReadFailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{RED}ERR{NORMAL}] Failed to read input")
    }
}

impl Error for ReadFailedError {}

use std::{env, error::Error};
use std::fmt::Display;
use std::io::Write;
use std::path::PathBuf;

use colors::{NORMAL, RED};
use interpreter::{Interpreter, Visitor};
use interpreter::resolver::Resolver;
use sourcemap::Source;
use syntax::tokenizer::Scanner;
use syntax::parser::Parser;

pub mod colors;
pub mod span;
pub mod sourcemap;
pub mod util;
pub mod interpreter;
pub mod syntax;

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

        self.run(&input);

        if self.static_error {
            std::process::exit(65);
        }

        if self.runtime_error {
            std::process::exit(70);
        }
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


            self.run(&line);

            print_prompt();
        }
    }

    pub fn run(&mut self, input: &str) {
        let source = Source::new(input);

        // Tokenizing
        let mut scanner = Scanner::new(&source);

        // Parsing
        let mut parser = Parser::new(&source, &mut scanner);
        let parsed = parser.parse();

        let ast = match parsed {
            Ok(ast) => ast,
            Err(_) => {
                return;
            }
        };

        // Variable resolution
        let mut resolver = Resolver::new(&source);
        let _ = resolver.visit(&ast);

        // Interpreting
        let mut interpreter = Interpreter::new(&source, resolver.locals);

        match interpreter.visit(&ast) {
            Ok(lit) => println!("{lit}"),
            Err(error) => {
                self.runtime_error = true;
                let annotated = source.annotate(error);
                eprintln!("{}", annotated);
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

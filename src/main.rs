use std::{env, error::Error};
use std::fmt::Display;
use std::io::Write;
use std::path::PathBuf;

use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use sourcemap::SourceMap;
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
pub mod interpreter;
pub mod environment;
pub mod errors;
pub mod sourcemap;
pub mod functions;
pub mod resolver;
pub mod util;
pub mod class;


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
        let sourcemap = SourceMap::new(input);

        // Tokenizing
        let mut scanner = Scanner::new(input);
        let tokens: Vec<Token> = scanner.by_ref().collect();

        // Move this up, somewhere else?
        for error in scanner.errors {
            self.static_error = true;
            let annotated = sourcemap.annotate(error);
            eprintln!("{}", annotated);
        }

        // Parsing
        let mut parser = Parser::new(tokens);
        let parsed = parser.parse();

        // Error reporting


        let ast = match parsed {
            Ok(ast) => ast,
            Err(errors) => {
                for error in errors {
                    self.static_error = true;
                    let annotated = sourcemap.annotate(error);
                    eprintln!("{}", annotated);
                }

                return;
            }
        };

        // Variable resolution
        let mut resolver = Resolver::new();
        resolver.resolve_ast(&ast);

        // Interpreting
        let mut interpreter = Interpreter::new(resolver.locals);

        match interpreter.interpret(&ast) {
            Ok(lit) => println!("{lit}"),
            Err(error) => {
                self.runtime_error = true;
                let annotated = sourcemap.annotate(error);
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

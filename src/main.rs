use std::{env, error::Error};
use std::fmt::Display;
use std::io::Write;
use std::path::PathBuf;

use tokenizer::Scanner;
use colors::{NORMAL, RED};

pub mod scanner;
pub mod colors;
pub mod ast;
pub mod pretty_print;
pub mod tokenizer;
pub mod tokens;
pub mod span;

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

    pub fn run(&mut self, input: &str) -> Result<(), &str> {
        let mut scanner = Scanner::new(input);

        for token in scanner.by_ref() {
            println!("{token:?}");
        }

        for error in scanner.errors() {
            eprintln!("[{RED}ERR{NORMAL}] {}", error.msg);
        }

        Ok(())
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

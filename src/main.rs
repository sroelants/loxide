use std::env;
use std::fmt::Display;
use std::io::Write;
use std::path::PathBuf;

use scanner::Token;
use colors::{NORMAL, RED};

mod scanner;
mod colors;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: loxide [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(file: &str) {
    let Ok(input) = std::fs::read_to_string(PathBuf::from(file)) else {
        eprintln!("[{RED}ERR{NORMAL}]: File not found: {file}");
        return;
    };

    if let Err(err) = run(&input) {
        eprintln!("{err}");
    };
}

fn run_prompt() {
    print!("> ");
    std::io::stdout().flush().unwrap();

    for line in std::io::stdin().lines() {
        let Ok(line) = line else {
            eprintln!("[{RED}ERR{NORMAL}] Failed to read input");
            print!("> ");
            std::io::stdout().flush().unwrap();
            continue;
        };


        if let Err(err) = run(&line) {
            eprintln!("{err}");
        };
    }
}

fn run(input: &str) -> Result<(), LoxideError> {
    println!("{input}");

    Ok(())
}

pub struct LoxideError<T> {
    token: Token,
    msg: &'static str,
}

impl Display for LoxideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{RED}ERR{NORMAL}] {line}:{col}: {msg}", line = self.token.line, col = self.token.col, msg = self.msg)
    }
}

extern crate clap;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;
mod lexer;

use clap::{App, Arg};
use std::fs;

fn main() {
    let matches = App::new("Anvil")
        .version("0.1")
        .about("Rust interpreter for Arn")
        .arg(Arg::from_usage("<file> 'The file to be interpreted'"))
        .get_matches();

    if let Some(path) = matches.value_of("file") {
        let program = read_file(path);
        println!("{:?}", lexer::expr_to_postfix(&lexer::lex(&program)));
    }
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).expect(&format!("\nFile '{}' does not exist.\n", path))
}

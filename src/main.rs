extern crate clap;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod ast;
mod lexer;
mod parser;
mod utils;

use clap::{App, Arg};
use std::fs;

// This is really cursed, but it works so hey
lazy_static! {
    pub static ref MATCHES: clap::ArgMatches<'static> = App::new("Anvil")
        .version("0.1")
        .about("Rust interpreter for Arn")
        .arg(Arg::from_usage("<file> 'The file to be interpreted'"))
        .arg(
            Arg::with_name("precision")
                .short("p")
                .long("precision")
                .value_name("INTEGER")
                .help("Precision of internal floats")
                .default_value("50"),
        )
        .arg(
            Arg::with_name("output-precision")
                .short("o")
                .long("oprecision")
                .value_name("INTEGER")
                .help("Determines precision of outputted numbers")
                .default_value("4")
        )
        .get_matches();
    pub static ref FLOAT_PRECISION: u32 = MATCHES.value_of("precision").unwrap().parse().unwrap();
    pub static ref OUTPUT_PRECISION: usize = MATCHES
        .value_of("output-precision")
        .unwrap()
        .parse()
        .unwrap();
}

fn main() {
    if let Some(path) = MATCHES.value_of("file") {
        let program = read_file(path);
        parser::parse(&ast::to_ast(&lexer::expr_to_postfix(&lexer::lex(&program))));
    }
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("\nFile '{}' does not exist.\n", path))
}

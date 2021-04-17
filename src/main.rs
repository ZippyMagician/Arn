#![deny(rust_2018_idioms, clippy::all)]
#![deny(mutable_borrow_reservation_conflict, clippy::clone_on_ref_ptr)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    // I have a lot of TODOs, so this is reduntant
    clippy::match_same_arms,
    // I don't care about this
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    // It's a ******* Rc
    clippy::needless_pass_by_value
)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod ast;
mod lexer;
mod parser;
mod utils;

use std::fs;

use clap::{App, Arg};

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
        let tree = ast::to_ast(&lexer::to_postfix(&lexer::lex(&program)));
        // Create thread to run parser in that features much larger stack
        let builder = std::thread::Builder::new()
            .name("parser".into())
            .stack_size(32 * 1024 * 1024);

        let handler = builder.spawn(move || parser::parse(&tree)).unwrap();
        handler.join().unwrap();
    }
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("\nFile '{}' does not exist.\n", path))
}

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
    pub static ref MATCHES: clap::ArgMatches<'static> = App::new("Arn")
        .version("1.0")
        .author("ZippyMagician <zippymagician1@gmail.com>")
        .about("The Rust interpreter for Arn")
        .arg(
            Arg::with_name("file")
                .required(true)
                .help("The input file to be run")
        )
        .arg(
            Arg::with_name("precision")
                .short("p")
                .long("precision")
                .help("Precision of internal floats")
                .takes_value(true)
                .value_name("INTEGER")
        )
        .arg(
            Arg::with_name("output-precision")
                .short("o")
                .long("oprecision")
                .help("Precision of outputted numbers")
                .takes_value(true)
                .value_name("INTEGER")
        )
        .arg(
            Arg::with_name("stack-size")
                .long("stack")
                .help("Sets the size of the allocated stack for the program")
                .takes_value(true)
                .value_name("MEGABYTES")
                .default_value("2")
        )
        .arg(
            Arg::with_name("one-ten")
                .short("d")
                .help("Sets STDIN to the range [1, 10]")
        )
        .arg(
            Arg::with_name("one-hundred")
                .short("h")
                .help("Sets STDIN to the range [1, 100]")
        )
        .arg(
            Arg::with_name("array")
                .short("a")
                .help("Wraps program in `[ ... ]`")
        )
        .arg(
            Arg::with_name("rangeify")
                .short("r")
                .help("Converts STDIN `r` to the range [1, r]")
        )
        .arg(
            Arg::with_name("map")
                .short("m")
                .help("Executes the program like it's mapped over the input")
        )
        .arg(
            Arg::with_name("first")
                .short("f")
                .help("Returns first value in return value of program")
        )
        .arg(
            Arg::with_name("last")
                .short("l")
                .help("Returns last value in return value of program")
        )
        .arg(
            Arg::with_name("size")
                .short("s")
                .help("Returns size of return value of program")
        )
        .arg(
            Arg::with_name("sum")
                .short("x")
                .help("Sums the returned value of the program")
        )
        .get_matches();
    pub static ref FLOAT_PRECISION: u32 = MATCHES
        .value_of("precision")
        .unwrap_or("50")
        .parse()
        .unwrap();
    pub static ref OUTPUT_PRECISION: usize = MATCHES
        .value_of("output-precision")
        .unwrap_or("4")
        .parse()
        .unwrap();
}

fn main() {
    if let Some(path) = MATCHES.value_of("file") {
        let mut program = read_file(path);
        let size = MATCHES.value_of("stack-size").unwrap().parse::<usize>().unwrap();

        // Some ARGV handling
        if MATCHES.is_present("array") {
            program = format!("[{}]", program);
        }
        if MATCHES.is_present("map") {
            program = format!("{{{}}}\\", program);
        }

        // Create thread to run parser in that features much larger stack
        let builder = std::thread::Builder::new()
            .name("parser".into())
            .stack_size(size * 1024 * 1024);

        let handler = builder.spawn(move || parser::parse(&build_ast(&program))).unwrap();
        handler.join().unwrap();
    }
}

#[inline]
pub fn build_ast(prg: &str) -> Vec<utils::tokens::Node> {
    ast::to_ast(&lexer::to_postfix(&lexer::lex(prg)))
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("\nFile '{}' does not exist.\n", path))
}

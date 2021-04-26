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

use std::fmt::Write;
use std::fs;

use clap::{App, Arg};

use crate::utils::compress;

// This is really cursed, but it works so hey
lazy_static! {
    pub static ref MATCHES: clap::ArgMatches<'static> = App::new("Arn")
        .version("1.1.1")
        .author("ZippyMagician <zippymagician1@gmail.com>")
        .about("The Rust interpreter for Arn")
        .arg(
            Arg::with_name("file")
                .required(true)
                .help("The input file to be run")
        )
        .arg(
            Arg::with_name("gen-answer")
                .long("cgans")
                .help("Generates a sample answer from the provided program for https://codegolf.stackexchange.com")
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
            Arg::with_name("input")
                .long("user-input")
                .short("u")
                .takes_value(true)
                .value_name("STDIN")
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
            Arg::with_name("compress")
                .short("c")
                .long("compress")
                .help("The input will be compressed and printed to STDOUT")
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("Prints some debug information (to help check if what you found was a bug or not)")
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
        .arg(
            Arg::with_name("index")
                .short("i")
                .help("Yields the i'th value in the return value of the program, where i is STDIN")
        )
        .arg(
            Arg::with_name("find")
                .short("I")
                .help("Gets the index of the input inside the return value of the program")
        )
        .arg(
            Arg::with_name("not")
                .short("!")
                .help("Boolean nots the returned value")
        )
        .arg(
            Arg::with_name("0-range")
                .short("R")
                .help("Sets STDIN N to the range [0, N)")
        )
        .arg(
            Arg::with_name("flat")
                .short("F")
                .help("Flattens returned value")
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
        // Read file, remove CRLF
        let mut program = read_file(path).replace("\r\n", "\n");

        if MATCHES.is_present("gen-answer") {
            let comp_program = compress::pack(&program);
            let mut flags = String::new();
            print!("# [Arn](https://github.com/ZippyMagician/Arn)");
            for arg in std::env::args()
                .skip(1)
                .filter(|h| !h.starts_with("--") && h.starts_with('-'))
                .flat_map(|n| n.trim_matches('-').chars().collect::<Vec<_>>())
            {
                if arg != 'p' && arg != 'o' {
                    write!(flags, "{}", arg).unwrap();
                }
            }

            if !flags.is_empty() {
                print!(" `-{}`", flags);
            }
            println!(
                ", [{} bytes](https://github.com/ZippyMagician/Arn/wiki/Carn)\n",
                comp_program.chars().count()
            );
            println!("```\n{}\n```\n", comp_program);
            println!(
                "# Explained\nUnpacked: `{}`\n```\nELABORATE HERE\n```",
                program
            );
            std::process::exit(0);
        }

        let size = MATCHES
            .value_of("stack-size")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        if MATCHES.is_present("compress") {
            println!("{}", compress::pack(&program));
            std::process::exit(0);
        }

        if compress::is_packed(&program) {
            program = compress::unpack(&program);
        }

        // Some ARGV handling
        if MATCHES.is_present("array") {
            program = format!("[{}]", program);
        }
        if MATCHES.is_present("map") {
            program = format!("{{{}}}\\", program);
        }
        if MATCHES.is_present("flat") {
            program = format!("({}):_", program);
        }
        if MATCHES.is_present("find") {
            program = format!("({}):i", program);
        }

        // Create thread to run parser in that features much larger stack
        let builder = std::thread::Builder::new()
            .name("parser".into())
            .stack_size(size * 1024 * 1024);

        let handler = builder
            .spawn(move || {
                if MATCHES.is_present("debug") {
                    println!("lexed: {:?}", lexer::lex(&program));
                    println!("ast: {:?}", build_ast(&program));
                }
                parser::parse(&build_ast(&program))
            })
            .unwrap();
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

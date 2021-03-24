extern crate clap;
#[macro_use]
extern crate lazy_static;

mod tokens;
#[macro_use]
mod macros;
mod consts;
mod lexer;
mod stream;

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
        println!("{}", lexer::to_postfix(&stream::insert_implied(&program)));
    }
}

fn read_file(path: &str) -> String {
    let err_msg = format!("\nFile '{}' does not exist.\n", path);

    fs::read_to_string(path).expect(&*err_msg)
}

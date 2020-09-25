extern crate clap;

pub(crate) mod deque;
pub(crate) mod tokens;
#[macro_use]
pub(crate) mod macros;

mod lexer;

use clap::{App, Arg};
use std::fs;

fn main() {
    let matches = App::new("Anvil")
        .version("1.0.0")
        .about("A compiled language")
        .arg(Arg::from_usage("<file> 'The file to be complied'"))
        .get_matches();

    if let Some(path) = matches.value_of("file") {
        let mut program = read_file(path);
        println!("{:?}", lexer::tokenize(&mut program));
    }
}

fn read_file(path: &str) -> String {
    let err_msg = format!("\nFile '{}' does not exist.\n", path);

    fs::read_to_string(path).expect(&*err_msg)
}

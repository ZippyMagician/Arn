extern crate clap;

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

#[test]
fn test_lexer() {
    use tokens::Node;

    assert_eq!(
        lexer::tokenize(&mut "3 + 4 * 2".to_string()),
        vec![Node::Fix(
            String::from("+"),
            vec![Node::Number(3)],
            vec![Node::Fix(
                String::from("*"),
                vec![Node::Number(4)],
                vec![Node::Number(2)]
            )]
        )]
    );

    assert_eq!(
        lexer::tokenize(&mut "6 ^ 2 / (3 + 7) * 2".to_string()),
        vec![Node::Fix(
            String::from("*"),
            vec![Node::Fix(
                String::from("/"),
                vec![Node::Fix(
                    String::from("^"),
                    vec![Node::Number(6)],
                    vec![Node::Number(2)]
                )],
                vec![Node::Fix(
                    String::from("+"),
                    vec![Node::Number(3)],
                    vec![Node::Number(7)]
                )]
            )],
            vec![Node::Number(2)]
        )]
    )
}

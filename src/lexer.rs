// Based on the Shunting Yard algorithm, modified to return an AST
// in a non-recursive manner.

use crate::deque::{Deque, Position};
use crate::tokens::Node;

// Little macro I created to make the global Operators class much nicer.
// First number is precedence, second is left # of args, third is right # of args
operators! {
    '!': 5; 0-1,
    '^': 4; 1-1,
    '*': 3; 1-1, '/': 3; 1-1,
    '+': 2; 1-1, '-': 2; 1-1
}

fn push_args(
    op: &String,
    left: &mut Vec<Node>,
    right: &mut Vec<Node>,
    control: &mut Deque<Node>,
    options: &Operators,
) {
    let mut li = 0;
    let mut ri = 0;

    while li < options.rank.get(op).unwrap().0 {
        left.push(control.dequeue(Position::BACK).unwrap_or(default!()));
        li += 1;
    }

    while ri < options.rank.get(op).unwrap().1 {
        right.push(control.dequeue(Position::BACK).unwrap_or(default!()));
        ri += 1;
    }
}

pub fn tokenize<'a>(code: &'a mut String) -> Vec<Node> {
    let options: Operators = Operators::new();
    // This enables the parsing to work properly
    code.push('\n');

    let mut done = false;

    let mut index = 0;

    let bytes = unsafe { code.as_bytes_mut() };
    bytes.rotate_right(1);
    bytes.reverse();

    let mut next = || -> Option<&u8> {
        index += 1;

        bytes.get(index - 1)
    };

    let mut control: Deque<Node> = Deque::with_allocation(20);
    let mut operators: Deque<String> = Deque::with_allocation(20);

    let mut buf = String::new();

    let mut in_string: bool = false;

    while !done {
        match next() {
            Some(val) => {
                let tok = *val as char;
                if buf == "\"" {
                    in_string = true;
                }

                if tok != ' ' || in_string {
                    // If the buffer is current a string
                    if in_string {
                        if tok == '"' {
                            buf.push(tok);
                            let mut buf_arr: Vec<&str> = buf.split("").collect();
                            buf_arr.reverse();

                            control.push_back(Node::String(buf_arr.join("")));

                            buf = String::new();
                            in_string = false
                        } else {
                            buf.push(tok);
                        }
                    // If the buffer is currently an integer
                    } else if let Ok(_) = buf.parse::<i128>() {
                        // And the top item of the stack is not an integer
                        if let Err(_) = tok.to_string().parse::<i128>() {
                            // Push the integer to the stack, reset the buffer, and push the new token
                            control.push_back(Node::Number(buf.parse().unwrap()));
                            buf = String::new();

                            buf.push(tok);
                        } else {
                            buf.push(tok);
                        }
                    // If the buffer is currently an operator
                    } else if options.operators.iter().any(|i| *i == buf) {
                        // And the current entry is also an operator
                        if options.operators.iter().any(|i| *i == tok.to_string()) {
                            buf.push(tok);
                            // And the current entry + the buffer is not an operator
                            if !options.operators.iter().any(|i| *i == buf) {
                                buf.pop();
                            }
                        }

                        while !operators.is_empty() {
                            let mut op = operators.get_top().clone();
                            if None == options.precedence.get(&buf)
                                || None == options.precedence.get(&op)
                                || options.precedence.get(&buf).unwrap()
                                    >= options.precedence.get(&op).unwrap()
                            {
                                break;
                            }

                            if options.operators.contains(&op) {
                                op = operators.dequeue(Position::BACK).unwrap();
                                let mut left: Vec<Node> = vec![];
                                let mut right: Vec<Node> = vec![];

                                push_args(&op, &mut left, &mut right, &mut control, &options);
                                control.push_back(Node::Fix(op, left, right));
                            }
                        }

                        operators.push_back(buf);
                        buf = String::new();
                        buf.push(tok);
                    // If the buffer is a left paren
                    } else if buf == "(" {
                        let end = operators.find_entry(')'.to_string()).unwrap_or_else(|| {
                            panic!("Unmatched parenthesis. Expected matching ')' in the program")
                        });

                        let mut i = operators.len() - 1;
                        while i > end && !operators.is_empty() {
                            let op = operators.dequeue(Position::BACK).unwrap();
                            let mut left: Vec<Node> = vec![];
                            let mut right: Vec<Node> = vec![];

                            push_args(&op, &mut left, &mut right, &mut control, &options);
                            control.push_back(Node::Fix(op, left, right));

                            i -= 1;
                        }

                        operators.remove(end);
                        buf = String::new();
                        buf.push(tok);
                    // The buffer is a right paren or right bracket
                    } else if buf == ")" {
                        operators.push_back(buf);
                        buf = String::new();
                        buf.push(tok);
                    // The buffer is a variable name
                    } else if buf.chars().all(char::is_alphanumeric) {
                        // If it's the end of a variable identifier
                        if !tok.is_alphanumeric() {
                            let mut buf_arr: Vec<&str> = buf.split("").collect();
                            buf_arr.reverse();

                            // Push to control stack and reset
                            control.push_back(Node::Variable(buf_arr.join("")));
                            buf = String::new();
                        }

                        buf.push(tok);
                    // If it is identified by the symbol operator (":")
                    } else if buf.chars().nth(0).unwrap_or(' ') == ':' {
                        // And the next character is also a symbol/integer
                        if tok == ':' || tok.is_numeric() {
                            buf.push(tok);
                        // Otherwise, the symbol is completed
                        } else {
                            let depth = buf
                                .chars()
                                .filter(|c| *c == ':')
                                .collect::<Vec<char>>()
                                .len();
                            let num_ident = buf.trim_start_matches(|c| c == ':').parse::<u8>();

                            control.push_back(Node::Symbol(
                                depth as u8,
                                num_ident.unwrap_or_else(|_| {
                                    panic!("Error parsing the symbol: {}", buf)
                                }),
                            ));
                        }
                    } else {
                        buf.push(tok);
                    }
                }
            }
            None => {
                done = true;

                while !operators.is_empty() {
                    if options.operators.contains(operators.get_top()) {
                        let op = operators.dequeue(Position::BACK).unwrap();
                        let mut left: Vec<Node> = vec![];
                        let mut right: Vec<Node> = vec![];

                        push_args(&op, &mut left, &mut right, &mut control, &options);
                        control.push_back(Node::Fix(op, left, right));
                    } else {
                        // Discard if invalid
                        operators.dequeue(Position::BACK);
                    }
                }
            }
        }
    }

    control.to_vec()
}

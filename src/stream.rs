use crate::consts::OPTIONS;
use crate::tokens::Node;

// TODO: This function will be modified once I need to implement `\`, `@`, and `;`.
// This lexes as a first pass, so I just change how groups / blocks are parsed and then grab 
// tokens for `\` to use (and others). I then need to represent it in string form in such a way 
// that the postfix parser can recognize it (probably make fold-ops represented as a string arg)
pub fn reformat_program(prg: &str) -> String {
    // Lets assume only 5 `_` will be inserted, this should help performance
    let mut construct: Vec<Node> = Vec::new();
    let mut output = String::with_capacity(prg.len() + 5);
    let mut buf: String = String::new();

    let mut in_string = false;
    let mut in_group: bool = false;

    for tok in prg.chars().chain("\n".chars()) {
        if buf == "\"" {
            in_string = true;
        }

        if !in_string && !in_group && (buf == "\n" || buf == " " || buf == "\r") {
            buf.clear();
        }

        if in_string {
            if tok == '"' {
                buf.push(tok);
                construct.push(Node::String(buf.clone()));
                buf.clear();
                in_string = false
            } else {
                buf.push(tok);
            }
        } else if in_group {
            if tok == ')' && buf.starts_with('(') {
                buf.push(tok);
                construct.push(Node::Literal(buf.clone()));
                buf.clear();
                in_group = false;
            } else if tok == '}' && buf.starts_with('{') {
                buf.push(tok);
                construct.push(Node::Literal(buf.clone()));
                buf.clear();
                in_group = false;
            } else {
                buf.push(tok);
            }
        } else if buf.parse::<i128>().is_ok() {
            if tok.to_string().parse::<i128>().is_err() {
                construct.push(Node::Number(buf.parse().unwrap()));
                buf.clear();
                buf.push(tok);
            } else {
                buf.push(tok);
            }
        } else if OPTIONS.operators.iter().any(|i| *i == buf) {
            buf.push(tok);
            let mut consumed = true;
            if !OPTIONS.operators.iter().any(|i| *i == buf) {
                buf.pop();
                consumed = false;
            }

            let rank = OPTIONS.rank.get(&buf).unwrap();
            if rank.0 > 0 {
                if construct.is_empty() {
                    for _ in 0..rank.0 {
                        construct.push(Node::Variable('_'.to_string()));
                    }
                } else if let Some(Node::Operator(_, stack_rank)) = construct.last() {
                    for _ in 0..stack_rank.1 {
                        construct.push(Node::Variable('_'.to_string()));
                    }
                }
            }

            construct.push(Node::Operator(buf.clone(), *rank));

            buf.clear();
            if !consumed {
                buf.push(tok);
            }
        } else if buf.starts_with(|c| c == '{' || c == '(') {
            in_group = true;
            buf.push(tok);
        } else if buf.chars().all(char::is_alphanumeric) && !buf.is_empty() {
            if !tok.is_alphanumeric() {
                construct.push(Node::Variable(buf.clone()));
                buf.clear();
            }

            buf.push(tok);
        } else {
            buf.push(tok);
        }
    }

    // If last op is missing args, push `_`
    if let Some(Node::Operator(_, rank)) = construct.last() {
        for _ in 0..rank.1 {
            construct.push(Node::Variable('_'.to_string()));
        }
    }

    for node in construct {
        output.push_str(&format!("{} ", node));
    }

    output
}

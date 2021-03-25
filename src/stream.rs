use crate::utils::consts::OPTIONS;
use crate::utils::tokens::Node;

pub fn reformat_program(prg: &str) -> String {
    // Lets assume only 5 `_` will be inserted, this should help performance
    let mut construct: Vec<Node> = Vec::new();
    let mut output = String::with_capacity(prg.len() + 5);
    let mut buf: String = String::new();

    let mut in_string = false;
    let mut in_group: bool = false;
    let mut group_count: usize = 0;
    let mut group_char: Option<char> = None;

    let bytes = prg.chars().chain("\n".chars());

    for tok in bytes {
        if buf == "\"" {
            in_string = true;
        }

        if !in_string && !in_group && (buf == "\n" || buf == " " || buf == "\r") {
            buf.clear();
        }

        if in_group && group_char.unwrap() == tok {
            group_count += 1;
        }

        if !in_group && (buf == "{" || buf == "(") {
            in_group = true;
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
                if group_count > 0 {
                    group_count -= 1;
                    buf.push(tok);
                } else {
                    buf.push(tok);
                    construct.push(Node::Literal(reformat_program(&buf[1..buf.len() - 1]), '('));
                    buf.clear();
                    in_group = false;
                }
            } else if tok == '}' && buf.starts_with('{') {
                if group_count > 0 {
                    group_count -= 1;
                    buf.push(tok);
                } else {
                    buf.push(tok);
                    construct.push(Node::Literal(reformat_program(&buf[1..buf.len() - 1]), '{'));
                    buf.clear();
                    in_group = false;
                }
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
        } else if buf.chars().all(char::is_alphanumeric) && !buf.is_empty() {
            if !tok.is_alphanumeric() {
                construct.push(Node::Variable(buf.clone()));
                buf.clear();
            }

            buf.push(tok);
        } else if tok == '{' || tok == '(' {
            if tok == '{' {
                group_char = Some('{');
            } else {
                group_char = Some('(');
            }

            buf.push_str(&format!("{} ", tok));
            in_group = true;
        // Not marked as group, need to do so
        } else {
            buf.push(tok);
        }
    }

    // If last op is missing args, push `_`
    let pos = construct
        .iter()
        .rposition(|n| match n {
            Node::Operator(_, _) => true,
            _ => false,
        })
        .unwrap_or(0);
    if let Some(Node::Operator(_, rank)) = construct.get(pos) {
        let given: usize = construct.len() - pos - 1;
        for _ in 0..rank.1 - given as i32 {
            construct.push(Node::Variable('_'.to_string()));
        }
    }

    for node in construct {
        output.push_str(&format!("{} ", node));
    }

    output
}

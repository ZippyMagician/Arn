use crate::utils::consts::OPTIONS;
use crate::utils::tokens::Token;
use crate::utils::num;

// Takes the inputted program and converts it into a stream of tokens
// Inserts the implied variable `_` wherever it is used
// TODO: This will handle literals used by `\` and `@` in the future - make modular instead of hardcode?
pub fn lex(prg: &str) -> Vec<Token> {
    // Lets assume only 5 `_` will be inserted, this should help performance
    let mut construct: Vec<Token> = Vec::new();
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
                construct.push(Token::String(buf.clone()));
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
                    construct.push(Token::Block(lex(&buf[1..buf.len() - 1]), '('));
                    buf.clear();
                    in_group = false;
                }
            } else if tok == '}' && buf.starts_with('{') {
                if group_count > 0 {
                    group_count -= 1;
                    buf.push(tok);
                } else {
                    buf.push(tok);
                    construct.push(Token::Block(lex(&buf[1..buf.len() - 1]), '{'));
                    buf.clear();
                    in_group = false;
                }
            } else {
                buf.push(tok);
            }
        } else if buf == "_" || num::is_arn_num(&buf) {
            buf.push(tok);
            if !num::is_arn_num(&buf) {
                buf.pop();
                construct.push(Token::Number(num::parse_arn_num(&buf).expect("Error parsing number")));
                buf.clear();
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
                        construct.push(Token::Variable('_'.to_string()));
                    }
                } else if let Some(Token::Operator(_, stack_rank)) = construct.last() {
                    for _ in 0..stack_rank.1 {
                        construct.push(Token::Variable('_'.to_string()));
                    }
                }
            }

            construct.push(Token::Operator(buf.clone(), *rank));

            buf.clear();
            if !consumed {
                buf.push(tok);
            }
        } else if buf.chars().all(char::is_alphabetic) && !buf.is_empty() {
            if !tok.is_alphanumeric() {
                construct.push(Token::Variable(buf.clone()));
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
        .rposition(|n| matches!(n, Token::Operator(_, _)))
        .unwrap_or(0);
    if let Some(Token::Operator(_, rank)) = construct.get(pos) {
        let given: usize = construct.len() - pos - 1;
        for _ in 0..rank.1 - given as i32 {
            construct.push(Token::Variable('_'.to_string()));
        }
    }

    construct
}

// Instead of parsing directly to an AST, I'll try this, which converts to postfix first. A second pass from another function that converts postfix to an ast is trivial.
// Uses the Shutning Yard Algorithm, parses a single expression
pub fn expr_to_postfix(tokens: &[Token]) -> Vec<Token> {
    // This enables the parsing to work properly
    let mut operators: Vec<Token> = Vec::with_capacity(20);
    let mut output = Vec::with_capacity(tokens.len());

    for tok in tokens {
        if let Token::Operator(left, rank) = tok {
            while !operators.is_empty() {
                let op = operators.pop().unwrap();
                if let Token::Operator(ref right, _) = op {
                    if OPTIONS.precedence.get(left).is_none()
                        || OPTIONS.precedence.get(right).is_none()
                        || OPTIONS.precedence.get(left).unwrap()
                            >= OPTIONS.precedence.get(right).unwrap()
                        || rank.0 == 0
                    {
                        operators.push(op);
                        break;
                    }
                }

                // The above will hold true
                output.push(op)
            }

            operators.push(tok.clone());
        } else if let Token::Block(body, ch) = tok {
            output.push(Token::Punctuation(*ch));

            for op in expr_to_postfix(&body) {
                output.push(op);
            }

            output.push(Token::Punctuation(match ch {
                '{' => '}',
                '(' => ')',
                _ => panic!("Unrecognized punc"),
            }));
        } else {
            output.push(tok.clone());
        }
    }

    while !operators.is_empty() {
        if let Token::Operator(_, _) = operators.last().unwrap() {
            output.push(operators.pop().unwrap());
        } else {
            // Discard if invalid
            operators.pop();
        }
    }

    output
}

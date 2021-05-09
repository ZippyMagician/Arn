use crate::utils::consts::OPTIONS;
use crate::utils::num;
use crate::utils::tokens::Token;

// Takes the inputted program and converts it into a stream of tokens
// Inserts the implied variable `_` wherever it is used
pub fn lex(prg: &str) -> Vec<Token> {
    let mut construct: Vec<Token> = Vec::new();
    let mut buf: String = String::new();

    let mut in_string = false;
    let mut in_group: bool = false;
    let mut group_count: usize = 0;
    let mut group_char: Option<char> = None;

    // Mark end of program with `→` (as that character is not supported in decompressed code)
    let bytes = prg.chars().chain("\u{2192}".chars());

    for tok in bytes {
        if buf == "\"" {
            in_string = true;
            buf.clear();
        }

        if !in_string
            && !in_group
            && (buf == "\n" || buf == " " || buf == "\r" || buf == "\u{2192}")
        {
            // \n is an implicit comma
            if buf == "\n" {
                construct.push(Token::Comma);
            }
            buf.clear();
        }

        // Why `as_ref`? I don't know. It doesn't work when it's `&&buf`
        if !in_group && ["{", "(", "[", "`", "'"].contains(&buf.as_ref()) {
            in_group = true;
            group_char = Some(buf.chars().next().unwrap());
        }

        if in_group && group_char.unwrap() == tok {
            let last = buf.chars().last().unwrap_or('\u{2192}');
            if group_char.unwrap() != '{' || (last != '.' && last != ':') {
                group_count += 1;
            }
        }

        if in_string {
            if tok == '"' && buf.ends_with('\\') {
                buf.drain(buf.len() - 1..);
                buf.push(tok);
            } else if tok == '"' || tok == '→' {
                construct.push(Token::String(buf.clone()));
                buf.clear();
                in_string = false
            } else {
                buf.push(tok);
            }
        } else if in_group {
            let last = buf.chars().last().unwrap_or('\u{2192}');
            if (tok == ')' || tok == '→') && Some('(') == group_char {
                if group_count > 0 {
                    group_count -= 1;
                    buf.push(tok);
                } else {
                    construct.push(Token::Block(lex(&buf[1..]), '(', None));
                    buf.clear();
                    in_group = false;
                }
            } else if (tok == '}' || tok == '→')
                && Some('{') == group_char
                && last != '.'
                && last != ':'
            {
                if group_count > 0 {
                    group_count -= 1;
                    buf.push(tok);
                } else {
                    if let Some(Token::Variable(name)) = construct.clone().last() {
                        construct.pop();
                        construct.push(Token::Block(lex(&buf[1..]), '{', Some(name.clone())));
                    } else {
                        construct.push(Token::Block(lex(&buf[1..]), '{', None));
                    }
                    buf.clear();
                    in_group = false;
                }
            } else if (tok == ']' || tok == '→') && Some('[') == group_char {
                if group_count > 0 {
                    group_count -= 1;
                    buf.push(tok);
                } else {
                    construct.push(Token::Block(lex(&buf[1..]), '[', None));
                    buf.clear();
                    in_group = false;
                }
            } else if (tok == '`' || tok == '→') && Some('`') == group_char {
                construct.push(Token::CmpString(buf[1..].to_owned(), '`'));
                buf.clear();
                in_group = false;
            } else if (tok == '\'' || tok == '→') && Some('\'') == group_char {
                construct.push(Token::CmpString(buf[1..].to_owned(), '\''));
                buf.clear();
                in_group = false;
            } else {
                buf.push(tok);
            }
        } else if buf == "_" || num::is_arn_num(&buf) {
            buf.push(tok);
            if !num::is_arn_num(&buf) {
                buf.pop();
                if buf == "_" {
                    construct.push(Token::Variable("_".to_string()));
                } else {
                    construct.push(Token::Number(
                        num::parse_arn_num(&buf).expect("Error parsing number"),
                    ));
                }
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

            // Insert mess of precedence logic here
            let rank = OPTIONS.rank.get(&buf).unwrap();
            if rank.0 > 0 {
                if construct.is_empty()
                    || construct.len() < rank.0 as usize
                    || construct.last() == Some(&Token::Comma)
                {
                    for _ in 0..rank.0 - construct.len() as i32
                        + if construct.last() == Some(&Token::Comma) {
                            construct.len() as i32
                        } else {
                            0
                        }
                    {
                        construct.push(Token::Variable('_'.to_string()));
                    }
                } else if let Some(Token::Operator(ident, stack_rank)) = construct
                    .iter()
                    .rfind(|m| matches!(m, Token::Operator(_, _)))
                {
                    let pos = construct
                        .iter()
                        .rposition(|m| matches!(m, Token::Operator(_, _)))
                        .unwrap();
                    // The previous op has a lower precedence or no right rank, shouldn't have `_` inserted after it yet
                    let used_rank = if OPTIONS.precedence.get(ident).unwrap()
                        < OPTIONS.precedence.get(&buf).unwrap()
                        && stack_rank.1 > 0
                    {
                        rank.0 as usize
                    } else {
                        stack_rank.1 as usize
                    };
                    if construct.len() - pos <= used_rank {
                        for _ in 0..used_rank - (construct.len() - pos - 1) {
                            construct.push(Token::Variable('_'.to_string()));
                        }
                    }
                }
            }

            construct.push(Token::Operator(buf.clone(), *rank));

            buf.clear();
            if !consumed {
                buf.push(tok);
            }
        } else if buf.chars().all(char::is_alphanumeric) && !buf.is_empty() {
            if !tok.is_alphanumeric() {
                construct.push(Token::Variable(buf.clone()));
                buf.clear();
            }

            buf.push(tok);
        } else if buf == "," {
            construct.push(Token::Comma);
            buf.clear();
            buf.push(tok);
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

pub fn to_postfix(tokens: &[Token]) -> Vec<Token> {
    let indexes = tokens.split(|t| t.clone() == Token::Comma);
    let mut output = Vec::new();

    for chunk in indexes {
        output.push(expr_to_postfix(chunk));
    }

    output.join(&[][..])
}

// Instead of parsing directly to an AST, I'll try this, which converts to postfix first. A second pass from another function that converts postfix to an ast is trivial.
// Uses the Shunting Yard Algorithm, parses a single expression
#[inline]
fn expr_to_postfix(tokens: &[Token]) -> Vec<Token> {
    // This enables the parsing to work properly
    let mut operators: Vec<Token> = Vec::with_capacity(20);
    let mut output = Vec::with_capacity(tokens.len());

    for tok in tokens {
        if let Token::Operator(right, rank) = tok {
            while !operators.is_empty() {
                let op = operators.pop().unwrap();
                if let Token::Operator(ref left, left_rank) = op {
                    if OPTIONS.precedence.get(left).is_none()
                        || OPTIONS.precedence.get(right).is_none()
                        || (OPTIONS.precedence.get(right).unwrap()
                            > OPTIONS.precedence.get(left).unwrap()
                            && left_rank.1 > 0)
                        || rank.0 == 0
                    {
                        operators.push(op);
                        break;
                    }

                    // If there are missing implied `_` from the program, insert them now. These will always be the last ones (so this won't lead to interpretation issues)
                    // The above will hold true
                    let total_rank =
                        crate::utils::sum_rank(left_rank.0 as i128 + left_rank.1 as i128, &output);
                    for _ in 0..(total_rank - output.len() as i128) {
                        output.push(Token::Variable('_'.to_string()));
                    }
                    output.push(op)
                }
            }

            operators.push(tok.clone());
        } else if let Token::Block(body, ch, nm) = tok {
            let new = to_postfix(&body);
            output.push(Token::Block(new, *ch, nm.clone()));
        } else {
            output.push(tok.clone());
        }
    }

    while !operators.is_empty() {
        if let Token::Operator(_, rank) = operators.last().unwrap() {
            let total_rank = crate::utils::sum_rank(rank.0 as i128 + rank.1 as i128, &output);
            for _ in 0..(total_rank - output.len() as i128) {
                output.push(Token::Variable('_'.to_string()));
            }
            output.push(operators.pop().unwrap());
        } else {
            // Discard if invalid
            operators.pop();
        }
    }

    output
}

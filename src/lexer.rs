// Based on the Shunting Yard algorithm

use crate::utils::consts::OPTIONS;

// Instead of parsing directly to an AST, I'll try this, which converts to postfix first. A second pass from another function that converts postfix to an ast is trivial.
pub fn to_postfix(code: &str) -> String {
    // This enables the parsing to work properly
    let bytes = code.chars().chain("\n".chars());

    let mut operators: Vec<String> = Vec::with_capacity(20);
    let mut output = String::with_capacity(code.len());

    let mut buf = String::new();

    let mut in_string = false;

    for tok in bytes {
        if buf == "\"" {
            in_string = true;
        }

        if !in_string && (buf == "\n" || buf == " " || buf == "\r") {
            buf.clear();
        }

        // If the buffer is current a string
        if in_string {
            if tok == '"' {
                buf.push(tok);
                output.push_str(&buf);
                output.push(' ');
                buf.clear();
                in_string = false
            } else {
                buf.push(tok);
            }
        // If the buffer is currently an integer
        } else if buf.parse::<i128>().is_ok() {
            // And the top item of the stream is not an integer
            if tok.to_string().parse::<i128>().is_err() {
                // Push the integer to the output, reset the buffer, and push the new token
                output.push_str(&buf);
                output.push(' ');
                buf.clear();
                buf.push(tok);
            } else {
                buf.push(tok);
            }
        // If the buffer is currently an operator
        } else if OPTIONS.operators.iter().any(|i| *i == buf) {
            buf.push(tok);
            let mut consumed = true;
            // See if the buffer + tok is an operator
            if !OPTIONS.operators.iter().any(|i| *i == buf) {
                buf.pop();
                consumed = false;
            }

            while !operators.is_empty() {
                let op = operators.pop().unwrap();
                if OPTIONS.precedence.get(&buf).is_none()
                    || OPTIONS.precedence.get(&op).is_none()
                    || OPTIONS.precedence.get(&buf).unwrap() >= OPTIONS.precedence.get(&op).unwrap()
                    || OPTIONS.rank.get(&buf).unwrap().0 == 0
                {
                    operators.push(op);
                    break;
                }

                if OPTIONS.operators.contains(&op) {
                    output.push_str(&op);
                    output.push(' ');
                } else {
                    operators.push(op);
                }
            }

            operators.push(buf.clone());
            buf.clear();
            if !consumed {
                buf.push(tok);
            }
        // If the buffer is a right paren
        } else if buf == ")" {
            let end = operators
                .iter()
                .position(|c| c == "(")
                .expect("Unmatched parenthesis. Expected matching '(' in the program");
            let mut i = operators.len() - 1;

            while i > end && !operators.is_empty() {
                let op = operators.pop().unwrap();
                output.push_str(&op);
                output.push(' ');
                i -= 1;
            }

            operators.remove(end);
            buf.clear();
            buf.push(tok);
        // Begin block
        } else if buf == "}" {
            let end = operators
                .iter()
                .position(|c| c == "{")
                .expect("Unmatched brackets. Expected '{' in the program to match '}'");
            let mut i = operators.len() - 1;

            while i > end && !operators.is_empty() {
                let op = operators.pop().unwrap();
                output.push_str(&op);
                output.push(' ');
                i -= 1;
            }
            output.push_str("} ");

            operators.remove(end);
            buf.clear();
            buf.push(tok);
        // The buffer is a left paren or bracket
        } else if buf == "(" || buf == "{" {
            operators.push(buf.clone());
            if buf == "{" {
                output.push_str("{ ");
            }

            buf.clear();
            buf.push(tok);
        // The buffer is a variable name
        } else if buf.chars().all(|c| c.is_alphanumeric() || c == '_') && !buf.is_empty() {
            // If it's the end of a variable identifier
            if !tok.is_alphanumeric() && tok != '_' {
                // Push to output
                output.push_str(&buf);
                output.push(' ');
                buf.clear();
            }

            buf.push(tok);
        } else {
            buf.push(tok);
        }
    }

    while !operators.is_empty() {
        if OPTIONS.operators.contains(&operators[operators.len() - 1]) {
            let op = operators.pop().unwrap();
            output.push_str(&op);

            output.push(' ');
        } else {
            // Discard if invalid
            operators.pop();
        }
    }

    output
}

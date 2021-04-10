use crate::utils::tokens::*;

pub fn to_ast(postfix: &[Token]) -> Vec<Node> {
    let mut output = Vec::with_capacity(postfix.len());

    for tok in postfix {
        match tok {
            Token::String(val) => output.push(Node::String(val.clone())),

            Token::Number(num) => output.push(Node::Number(num.clone())),

            Token::Variable(val) => output.push(Node::Variable(val.clone())),

            Token::Block(body, chr) => {
                output.push(match *chr {
                    '{' => Node::Block(to_ast(body)),
                    '(' => Node::Group(to_ast(body)),
                    _ => unimplemented!(),
                });
            }

            Token::Operator(ident, rank) => {
                let mut left = Vec::new();
                let mut right = Vec::new();

                for _ in 0..rank.1 {
                    // I can call unwrap as lexer::expr_to_postfix guarantees arguments exist
                    right.insert(0, output.pop().unwrap());
                }

                for _ in 0..rank.0 {
                    left.insert(0, output.pop().unwrap());
                }

                output.push(Node::Op(ident.clone(), left, right));
            }

            _ => panic!("Unrecognized node"),
        }
    }

    output
}

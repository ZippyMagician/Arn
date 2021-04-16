use crate::utils::num::Num;
use crate::utils::tokens::*;
use crate::FLOAT_PRECISION;

pub fn to_ast(postfix: &[Token]) -> Vec<Node> {
    let mut output = Vec::with_capacity(postfix.len());

    for tok in postfix {
        match tok {
            Token::String(val) => output.push(Node::String(val.clone())),

            Token::Number(num) => output.push(Node::Number(num.clone())),

            Token::Variable(val) => output.push(Node::Variable(val.clone())),

            Token::Block(body, chr, nm) => {
                output.push(match *chr {
                    '{' => Node::Block(to_ast(body), nm.clone()),
                    '(' => Node::Group(to_ast(body)),
                    '[' => {
                        let body = to_ast(body);
                        match body.last() {
                            // (Maybe) Sized Sequence
                            Some(Node::Op(f, block_node, size_node)) if f == "->" => {
                                if let Some(Node::Block(_, _)) = block_node.get(0) {
                                    let seq_body = &body[0..body.len() - 1];
                                    Node::Sequence(
                                        seq_body.to_owned(),
                                        Box::new(block_node[0].clone()),
                                        Some(Box::new(size_node[0].clone())),
                                    )
                                } else {
                                    Node::Sequence(
                                        body.clone(),
                                        Box::new(Node::Block(vec![], None)),
                                        Some(Box::new(Node::Number(Num::with_val(
                                            *FLOAT_PRECISION,
                                            body.len(),
                                        )))),
                                    )
                                }
                            }

                            // Infinite sequence
                            Some(Node::Block(_, _)) => {
                                let seq_body = &body[0..body.len() - 1];
                                Node::Sequence(
                                    seq_body.to_owned(),
                                    Box::new(body.last().unwrap().clone()),
                                    None,
                                )
                            }

                            // Constant sequence
                            _ => Node::Sequence(
                                body.clone(),
                                Box::new(Node::Block(vec![], None)),
                                Some(Box::new(Node::Number(Num::with_val(
                                    *FLOAT_PRECISION,
                                    body.len(),
                                )))),
                            ),
                        }
                    }
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

            _ => panic!("Error on token {:?}", tok),
        }
    }

    output
}

#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A fix is any operation (usually denoted by punctuation)
    /// that takes in arguments on the left and/or right.
    Fix(String, Vec<Node>, Vec<Node>),

    /// String Node
    String(String),

    /// Numeric Node
    Number(i128),

    /// Variable Node
    Variable(String),

    /// A block that contains some code
    Block(Vec<Node>),

    /// Used by stream::insert_implied
    Operator(String, (i32, i32)),

    /// Also used by stream::insert_implied
    Literal(String),

    /// An empty entry, to be removed
    Empty,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::String(s) => write!(f, "{}", s),
            Node::Number(s) => write!(f, "{}", s),
            Node::Variable(s) => write!(f, "{}", s),
            Node::Operator(s, _) => write!(f, "{}", s),
            Node::Literal(s) => write!(f, "{}", s),
            _ => unimplemented!(),
        }
    }
}

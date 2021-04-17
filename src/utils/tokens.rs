use super::num::Num;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// String Node
    String(String),

    /// Numeric Node
    Number(Num),

    /// Variable Node
    Variable(String),

    /// A block that contains some code
    Block(Vec<Token>, char, Option<String>),

    /// Operator
    Operator(String, (i32, i32)),

    /// Comma
    Comma,
}

// Will be used by ast once implemented
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A fix is any operation (usually denoted by punctuation)
    /// that takes in arguments on the left and/or right.
    Op(String, Vec<Node>, Vec<Node>),

    /// String Node
    String(String),

    /// Numeric Node
    Number(Num),

    /// Variable Node
    Variable(String),

    /// A Group `( ... )`
    Group(Vec<Node>),

    /// A Block `{ ... }`
    Block(Vec<Node>, Option<String>),

    /// A Sequence `[ ... ]`
    /// Body, block, size
    Sequence(Vec<Node>, Box<Node>, Option<Box<Node>>),
}

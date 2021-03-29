#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// String Node
    String(String),

    /// Numeric Node
    Number(i128),

    /// Variable Node
    Variable(String),

    /// A block that contains some code
    Block(Vec<Token>, char),

    /// Operator
    Operator(String, (i32, i32)),

    /// Punctuation
    Punctuation(char),
}

// Will be used by ast once implemented
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// An empty entry, to be removed
    Empty,
}

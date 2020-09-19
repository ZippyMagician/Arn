#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A fix is any operation (usually denoted by punctuation)
    /// that takes in arguments on the left and/or right.
    Fix(String, Vec<Node>, Vec<Node>),

    /// String Node
    String(String),

    /// Numeric Node
    Number(i128),

    // TODO: Function call
    // Function(String, Vec<Node>, Vec<Node>),
    /// Variable Node
    Variable(String),

    /// The symbol is a special variable; it represents the arguments of a block
    Symbol(u8, u8),

    /// An empty entry, to be removed
    Empty,
}

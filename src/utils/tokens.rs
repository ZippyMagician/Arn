use std::fmt::{self, Display, Formatter};

use super::num::Num;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// String Node
    String(String),

    /// Compressed String Node
    CmpString(String, char),

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

    /// Compressed String Node
    CmpString(String, char),

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

// Hacky but it'll do
impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Op(val, l, r) => {
                for item in l {
                    write!(f, "{}", item)?;
                }
                write!(f, "{} ", val)?;
                for item in r {
                    write!(f, "{}", item)?;
                }

                Ok(())
            }

            Self::String(st) => write!(f, "\"{}\" ", st),

            Self::CmpString(cst, chr) => {
                write!(f, "\"{}\"", super::dict::decompress(cst, *chr == '\''))
            }

            Self::Number(num) => write!(f, "{} ", super::types::Dynamic::from(num.clone())),

            Self::Variable(st) => write!(f, "{} ", st),

            Self::Group(nodes) => {
                write!(f, "(")?;
                for node in nodes {
                    write!(f, "{}", node)?;
                }
                write!(f, ")")
            }

            Self::Block(nodes, name) => {
                if let Some(name) = name {
                    write!(f, "{}", name)?;
                }
                write!(f, "{{")?;
                for node in nodes {
                    write!(f, "{}", node)?;
                }
                write!(f, "}} ")
            }

            Self::Sequence(entries, block, len) => {
                write!(f, "[")?;
                for entry in entries {
                    write!(f, "{}, ", entry)?;
                }
                write!(f, "{}", block.as_ref())?;

                if let Some(len) = len {
                    write!(f, "-> {}", len.as_ref())?;
                }

                write!(f, "]")
            }
        }
    }
}

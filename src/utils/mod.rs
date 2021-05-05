use self::tokens::*;

pub mod compress;
pub mod consts;
pub mod dict;
pub mod env;
pub mod num;
pub mod tokens;
pub mod types;

// Creates a space-separated range [s, n]
pub fn create_str_range(s: usize, n: usize) -> String {
    (s..=n).map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
}

// Sums the rank from the start and slice of Tokens
pub fn sum_rank(start: i128, rest: &[Token]) -> i128 {
    start
        + rest.iter().cloned().fold(0_i128, |acc, op| {
            if let Token::Operator(_, rank) = op {
                acc + rank.0 as i128 + rank.1 as i128
            } else {
                acc
            }
        })
}

// Systematically replace `_` with values from entries
pub fn traverse_replace(entries: &mut Vec<Node>, tree: Node) -> Node {
    match &tree {
        Node::Block(body, nm) => {
            let new_body = body
                .iter()
                .map(|n| traverse_replace(entries, n.clone()))
                .collect();
            Node::Block(new_body, nm.clone())
        }

        Node::String(_) => tree,

        Node::Number(_) => tree,

        Node::Variable(v) => {
            if v == "_" {
                entries
                    .pop()
                    .expect("Too many `_` found in block by utils::traverse_replace")
                    .clone()
            } else {
                tree
            }
        }

        Node::Group(body) => {
            let new_body = body
                .iter()
                .map(|n| traverse_replace(entries, n.clone()))
                .collect();
            Node::Group(new_body)
        }

        Node::Op(n, largs, rargs) => {
            let nl = largs
                .iter()
                .map(|n| traverse_replace(entries, n.clone()))
                .collect();
            let nr = rargs
                .iter()
                .map(|n| traverse_replace(entries, n.clone()))
                .collect();
            Node::Op(n.clone(), nl, nr)
        }

        _ => unimplemented!(),
    }
}

// Create non-base10 matrix
pub fn nbase_padded<T: FnMut(String) -> String>(
    mut orig: self::types::Dynamic,
    mut f: T,
) -> Vec<String> {
    if !orig.is_array() {
        orig = self::types::Dynamic::from([orig]);
    }
    // This should be inferred, but isn't
    let mut v: Vec<String> = orig
        .literal_array()
        .map(|d| f(d.literal_string()))
        .collect();
    let max = v.iter().map(|n| n.len()).max().unwrap();
    v = v
        .iter()
        .map(|n| format!("{:0>size$}", n, size = max))
        .collect();
    v
}

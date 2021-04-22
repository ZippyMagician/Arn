use self::tokens::Node;

pub mod compress;
pub mod consts;
pub mod env;
pub mod num;
pub mod tokens;
pub mod types;

pub fn create_str_range(s: usize, n: usize) -> String {
    (s..=n).map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
}

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

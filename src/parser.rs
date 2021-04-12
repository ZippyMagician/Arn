use crate::utils::{env::Environment, types::*, tokens::Node};
use crate::utils::num::{FLOAT_PRECISION, Num};

pub fn parse_node(env: &mut Environment, node: &Node) -> Dynamic {
    match node {
        Node::Op(_, _, _) => unimplemented!(),

        Node::String(v) => Dynamic::from(v.clone()),

        Node::Number(v) => Dynamic::from(v.clone()),

        Node::Variable(v) => {
            if let Some(val) = env.vars.get(v) {
                val.clone()
            } else {
                panic!("Unrecognized variable {}", v);
            }
        }

        Node::Group(_) => unimplemented!(),

        Node::Block(_) => unimplemented!(),
    }
}

pub fn parse(ast: &[Node]) {
    let mut env = Environment::init();
    env.define_var("E", Num::with_val(FLOAT_PRECISION, Num::parse("2.7182818284590452353602874713527").unwrap()));

    for node in &ast[..ast.len() - 1] {
        parse_node(&mut env, node);
    }

    let default = Node::String(String::new());
    println!("{}", parse_node(&mut env, ast.last().unwrap_or(&default)));
}
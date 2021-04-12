use crate::utils::num::{Num, FLOAT_PRECISION};
use crate::utils::{env::Environment, tokens::Node, types::*};

lazy_static! {
    static ref DEFAULT: Node = Node::String(Default::default());
}

pub fn parse_op(env: &mut Environment, op: &str, left: &[Node], right: &[Node]) -> Dynamic {
    match op {
        "." => {
            if let Node::Variable(v) = &right[0] {
                let arg = parse_node(env, &left[0]);
                env.attempt_call(v, arg)
            } else {
                panic!("Dot operator only accepts a variable on the right hand side")
            }
        }

        "^" => {
            let mut left = parse_node(env, &left[0]);
            if !left.is_string() {
                let mut left = left.literal_num();
                let o = left.clone();
                let right = parse_node(env, &right[0]).literal_num();
                for _ in 1..right
                    .to_u32_saturating_round(rug::float::Round::Down)
                    .unwrap()
                {
                    left *= o.clone();
                }

                Dynamic::from(left)
            } else {
                left.mutate_string(|s| s.repeat(parse_node(env, &right[0]).literal_num().to_u32_saturating_round(rug::float::Round::Down).unwrap() as usize));
                left
            }
        }

        "*" => {
            let mut left = parse_node(env, &left[0]).into_num();
            left.mutate_num(|n| n * parse_node(env, &right[0]).literal_num());
            left
        }

        "/" => {
            let mut left = parse_node(env, &left[0]).into_num();
            left.mutate_num(|n| n / parse_node(env, &right[0]).literal_num());
            left
        }

        "%" => {
            let mut left = parse_node(env, &left[0]).into_num();
            left.mutate_num(|n| n % parse_node(env, &right[0]).literal_num());
            left
        }

        ":|" => todo!("Sequences needed"),

        ":!" => todo!("Sequences needed"),

        "+" => {
            let mut left = parse_node(env, &left[0]).into_num();
            left.mutate_num(|n| n + parse_node(env, &right[0]).literal_num());
            left
        }

        "-" => {
            let mut left = parse_node(env, &left[0]).into_num();
            left.mutate_num(|n| n - parse_node(env, &right[0]).literal_num());
            left
        }

        "," => todo!("Sequences needed"),

        ".$" => todo!("Sequences needed"),

        "=>" => todo!("Sequences needed"),

        "->" => todo!("Sequences needed"),

        "~" => todo!("Sequences needed"),

        "#" => todo!("Sequences needed"),

        ":_" => todo!("Sequences needed"),

        ".@" => todo!("Sequences needed"),

        ".|" => {
            let mut left = parse_node(env, &left[0]).into_num();
            left.mutate_num(|n| n.abs());
            left
        }

        ".<" => todo!("Sequences needed"),

        ".." => todo!("Sequences needed"),

        ".=" => todo!("Sequences needed"),

        ":n" => todo!("Sequences needed"),

        ":s" => todo!("Sequences needed"),

        ":}" => todo!("Sequences needed"),

        ":{" => todo!("Sequences needed"),

        ".{" => todo!("Sequences needed"),

        ".}" => todo!("Sequences needed"),

        ":@" => todo!("Sequences needed"),

        "^*" => {
            let left = parse_node(env, &left[0]).literal_num();
            Dynamic::from(left > 0 && left.sqrt() % 1 == 0)
        }

        "&." => {
            if let Node::Block(_) = right[0] {
                let mut loop_arg = parse_node(env, &right[1]);
                let count = parse_node(env, &right[2])
                    .literal_num()
                    .to_u32_saturating_round(rug::float::Round::Down)
                    .unwrap();
                for _ in 0..count {
                    let mut child_env = env.clone();
                    child_env.define_var("_", loop_arg);
                    loop_arg = parse_node(&mut child_env, &right[0]);
                }

                loop_arg
            } else {
                panic!("First argument to `&.` must be Node::Block")
            }
        }

        _ => unimplemented!(),
    }
}

pub fn parse_node(env: &mut Environment, node: &Node) -> Dynamic {
    match node {
        Node::Op(op, left, right) => parse_op(env, op, left, right),

        Node::String(v) => Dynamic::from(v.clone()),

        Node::Number(v) => Dynamic::from(v.clone()),

        Node::Variable(v) => {
            if let Some(val) = env.vars.get(v) {
                val.clone()
            } else {
                panic!("Unrecognized variable {}", v);
            }
        }

        Node::Group(body) | Node::Block(body) => {
            for node in &body[..body.len() - 1] {
                parse_node(env, node);
            }

            parse_node(env, body.last().unwrap_or(&DEFAULT))
        }
    }
}

pub fn parse(ast: &[Node]) {
    let mut env = Environment::init();

    env.define_var(
        "E",
        Num::with_val(
            FLOAT_PRECISION,
            Num::parse("2.7182818284590452353602874713527").unwrap(),
        ),
    );

    env.define_fn("inc", |d| Dynamic::from(d.literal_num() + 1));

    for node in &ast[..ast.len() - 1] {
        parse_node(&mut env, node);
    }

    println!("{}", parse_node(&mut env, ast.last().unwrap_or(&DEFAULT)));
}

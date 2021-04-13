use std::io::{self, Read};

use crate::utils::num::Num;
use crate::utils::{env::Environment, tokens::Node, types::*};
use crate::FLOAT_PRECISION;

lazy_static! {
    static ref DEFAULT: Node = Node::String(Default::default());
    static ref USCORE: String = String::from("_");
}

pub fn parse_op(env: &mut Environment, op: &str, left: &[Node], right: &[Node]) -> Dynamic {
    match op {
        // <right>(<left>)
        "." => {
            if let Node::Variable(v) = &right[0] {
                let arg = parse_node(env, &left[0]);
                env.attempt_call(v, arg)
            } else {
                panic!("Dot operator only accepts a variable on the right hand side")
            }
        }

        // <left> pow <right>
        "^" => {
            let left = parse_node(env, &left[0]);
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
                left.mutate_string(|s| {
                    s.repeat(
                        parse_node(env, &right[0])
                            .literal_num()
                            .to_u32_saturating_round(rug::float::Round::Down)
                            .unwrap() as usize,
                    )
                })
            }
        }

        // <left> × <right>
        "*" => {
            let left = parse_node(env, &left[0]);
            left.mutate_num(|n| n * parse_node(env, &right[0]).literal_num())
        }

        // <left> ÷ <right>
        "/" => {
            let left = parse_node(env, &left[0]);
            left.mutate_num(|n| n / parse_node(env, &right[0]).literal_num())
        }

        // <left> mod <right>
        "%" => {
            let left = parse_node(env, &left[0]);
            left.mutate_num(|n| n % parse_node(env, &right[0]).literal_num())
        }

        ":|" => todo!("Sequences needed"),

        ":!" => todo!("Sequences needed"),

        // <left> + <right>
        "+" => {
            let left = parse_node(env, &left[0]);
            left.mutate_num(|n| n + parse_node(env, &right[0]).literal_num())
        }

        // <left> - <right>
        "-" => {
            let left = parse_node(env, &left[0]);
            left.mutate_num(|n| n - parse_node(env, &right[0]).literal_num())
        }

        ".$" => todo!("Sequences needed"),

        "=>" => todo!("Sequences needed"),

        "->" => todo!("Sequences needed"),

        "~" => todo!("Sequences needed"),

        "#" => todo!("Sequences needed"),

        ":_" => todo!("Sequences needed"),

        ".@" => todo!("Sequences needed"),

        // |<left>|
        ".|" => {
            let left = parse_node(env, &left[0]);
            left.mutate_num(|n| n.abs())
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

        // is <left> perfect square?
        "^*" => {
            let left = parse_node(env, &left[0]).literal_num();
            Dynamic::from(left.sqrt().is_integer())
        }

        // Repeat <r1> <r3> times with initial value <r2>
        "&." => {
            if let Node::Block(_, name) = &right[0] {
                let mut loop_arg = parse_node(env, &right[1]);
                let count = parse_node(env, &right[2])
                    .literal_num()
                    .to_u32_saturating_round(rug::float::Round::Down)
                    .unwrap();
                for _ in 0..count {
                    let mut child_env = env.clone();
                    child_env.define_var(name.as_ref().unwrap_or(&USCORE), loop_arg);
                    loop_arg = parse_node(&mut child_env, &right[0]);
                }

                loop_arg
            } else {
                panic!("First argument to `&.` must be Node::Block")
            }
        }

        ":i" => todo!("Sequences needed"),

        // not <right>
        "!" => Dynamic::from(!parse_node(env, &right[0]).literal_bool()),

        "$" => todo!("Sequences needed"),

        // Floor <right>
        ":v" => {
            let right = parse_node(env, &right[0]);
            right.mutate_num(|n| n.floor())
        }

        // Ceil <right>
        ":^" => {
            let right = parse_node(env, &right[0]);
            right.mutate_num(|n| n.floor())
        }

        // Inc <right>
        "++" => {
            if let Node::Variable(name) = &right[0] {
                let val = env.vars.get_mut(name).expect("Variable not recognized");
                *val = val.mutate_num(|n| n + 1);
                val.clone()
            } else {
                let right = parse_node(env, &right[0]);
                right.mutate_num(|n| n + 1)
            }
        }

        // Dec <right>
        "--" => {
            if let Node::Variable(name) = &right[0] {
                let val = env.vars.get_mut(name).expect("Variable not recognized");
                *val = val.mutate_num(|n| n - 1);
                val.clone()
            } else {
                let right = parse_node(env, &right[0]);
                right.mutate_num(|n| n - 1)
            }
        }

        // <right> ^ 2
        ":*" => parse_node(env, &right[0]).mutate_num(|n| n.square()),

        // √<right>
        ":/" => parse_node(env, &right[0]).mutate_num(|n| n.sqrt()),

        // 2<right>
        ":+" => parse_node(env, &right[0]).mutate_num(|n| n * 2),

        // ½<right>
        ":-" => parse_node(env, &right[0]).mutate_num(|n| n / 2),

        ":>" => todo!("Sequences needed"),

        ":<" => todo!("Sequences needed"),

        "|:" => todo!("Sequences needed"),

        "$:" => todo!("Sequences needed"),

        "?." => todo!("Sequences needed"),

        "#." => todo!("Sequences needed"),

        "*." => todo!("Sequences needed"),

        "$." => todo!("Sequences needed"),

        "z" => todo!("Sequences needed"),

        // Concat <left> and <right>
        "|" => {
            // TODO: special case if concatenating to array
            let mut left = parse_node(env, &left[0]).literal_string();
            left.push_str(&parse_node(env, &right[0]).literal_string());
            Dynamic::from(left)
        }

        // Very weakly typed, see `src/utils/types.rs`, PartialEq for Dynamic
        // <left> == <right>
        "=" => Dynamic::from(parse_node(env, &left[0]) == parse_node(env, &right[0])),

        // <left> != <right>
        "!=" => Dynamic::from(parse_node(env, &left[0]) != parse_node(env, &right[0])),

        // <left> < <right>
        "<" => Dynamic::from(
            parse_node(env, &left[0]).literal_num() < parse_node(env, &right[0]).literal_num(),
        ),

        // <left> <= <right>
        "<=" => Dynamic::from(
            parse_node(env, &left[0]).literal_num() <= parse_node(env, &right[0]).literal_num(),
        ),

        // <left> > <right>
        ">" => Dynamic::from(
            parse_node(env, &left[0]).literal_num() > parse_node(env, &right[0]).literal_num(),
        ),

        // <left> >= <right>
        ">=" => Dynamic::from(
            parse_node(env, &left[0]).literal_num() >= parse_node(env, &right[0]).literal_num(),
        ),

        // <left> && <right> yields <right> if both truthy
        "&&" => {
            let left = parse_node(env, &left[0]);
            let right = parse_node(env, &right[0]);

            if left.is_bool() {
                if right.is_bool() {
                    Dynamic::from(left.literal_bool() && right.literal_bool())
                } else if left.clone().literal_bool() && right.clone().literal_bool() {
                    right
                } else {
                    left
                }
            } else if right.is_bool() {
                Dynamic::from(left.literal_bool() && right.literal_bool())
            } else if left.clone().literal_bool() && right.clone().literal_bool() {
                right
            } else {
                left
            }
        }

        // <left> || <right> Yields <left> if truthy and <right> otherwise
        "||" => {
            let left = parse_node(env, &left[0]);
            let right = parse_node(env, &right[0]);

            if left.clone().literal_bool() {
                left
            } else if right.clone().literal_bool() {
                right
            } else {
                Dynamic::from(false)
            }
        }

        // Do <left> while <right> (left & right take previous left value as arg), yields final mutated value
        // TODO: separate fix that yields number of iterations?
        ":" => {
            // TODO: When blocks have different variable names, this will use a child env
            let mut child_env = env.clone();
            if let Node::Block(_, name) = &left[0] {
                child_env.define_var(
                    name.as_ref().unwrap_or(&USCORE),
                    child_env.vars.get("_").unwrap().clone(),
                )
            }
            let mut block = parse_node(&mut child_env, &left[0]);

            while {
                let mut child_env = env.clone();
                if let Node::Block(_, name) = &right[0] {
                    child_env.define_var(name.as_ref().unwrap_or(&USCORE), block.clone())
                } else {
                    child_env.define_var("_", block.clone());
                }
                parse_node(&mut child_env, &right[0]).literal_bool()
            } {
                let mut child_env = env.clone();
                if let Node::Block(_, name) = &left[0] {
                    child_env.define_var(name.as_ref().unwrap_or(&USCORE), block.clone())
                } else {
                    child_env.define_var("_", block.clone());
                }
                block = parse_node(&mut child_env, &left[0]);
            }

            block
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

        Node::Group(body) => {
            for node in &body[..body.len() - 1] {
                parse_node(env, node);
            }

            parse_node(env, body.last().unwrap_or(&DEFAULT))
        }

        Node::Block(body, name) => {
            let mut child_env = env.clone();
            child_env.define_var(
                name.as_ref().unwrap_or(&USCORE),
                child_env.vars.get("_").unwrap().clone(),
            );
            for node in &body[..body.len() - 1] {
                parse_node(&mut child_env, node);
            }

            parse_node(&mut child_env, body.last().unwrap_or(&DEFAULT))
        }
    }
}

pub fn parse(ast: &[Node]) {
    let mut env = Environment::init();

    let mut stdin = String::new();
    io::stdin()
        .read_to_string(&mut stdin)
        .expect("Could not read from stdin");

    env.define_var("_", stdin);
    env.define_var(
        "E",
        Num::with_val(
            *FLOAT_PRECISION,
            Num::parse("2.7182818284590452353602874713527").unwrap(),
        ),
    );

    env.define_fn("out", |d| {
        println!("{}", d);
        d
    });

    for node in &ast[..ast.len() - 1] {
        parse_node(&mut env, node);
    }

    println!("{}", parse_node(&mut env, ast.last().unwrap_or(&DEFAULT)));
}

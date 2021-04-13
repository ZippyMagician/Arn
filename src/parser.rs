use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;

use crate::utils::num::Num;
use crate::utils::{env::Environment, tokens::Node, types::*};
use crate::FLOAT_PRECISION;

lazy_static! {
    static ref DEFAULT: Node = Node::String(Default::default());
    static ref USCORE: String = String::from("_");
}

pub fn parse_op(env: Env, op: &str, left: &[Node], right: &[Node]) -> Dynamic {
    match op {
        // <right>(<left>)
        "." => {
            if let Node::Variable(v) = &right[0] {
                let arg = parse_node(env.clone(), &left[0]);
                env.borrow_mut().attempt_call(v, arg)
            } else {
                panic!("Dot operator only accepts a variable on the right hand side")
            }
        }

        // <left> pow <right>
        "^" => {
            let left = parse_node(env.clone(), &left[0]);
            if !left.is_string() {
                let mut left = left.literal_num();
                let o = left.clone();
                let right = parse_node(env.clone(), &right[0]).literal_num();
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
                        parse_node(env.clone(), &right[0])
                            .literal_num()
                            .to_u32_saturating_round(rug::float::Round::Down)
                            .unwrap() as usize,
                    )
                })
            }
        }

        // <left> × <right>
        "*" => {
            let left = parse_node(env.clone(), &left[0]);
            left.mutate_num(|n| n * parse_node(env.clone(), &right[0]).literal_num())
        }

        // <left> ÷ <right>
        "/" => {
            let left = parse_node(env.clone(), &left[0]);
            left.mutate_num(|n| n / parse_node(env.clone(), &right[0]).literal_num())
        }

        // <left> mod <right>
        "%" => {
            let left = parse_node(env.clone(), &left[0]);
            left.mutate_num(|n| n % parse_node(env.clone(), &right[0]).literal_num())
        }

        ":|" => todo!("Sequences needed"),

        ":!" => todo!("Sequences needed"),

        // <left> + <right>
        "+" => {
            let left = parse_node(env.clone(), &left[0]);
            left.mutate_num(|n| n + parse_node(env.clone(), &right[0]).literal_num())
        }

        // <left> - <right>
        "-" => {
            let left = parse_node(env.clone(), &left[0]);
            left.mutate_num(|n| n - parse_node(env.clone(), &right[0]).literal_num())
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
            let left = parse_node(env.clone(), &left[0]);
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
            let left = parse_node(env.clone(), &left[0]).literal_num();
            Dynamic::from(left.sqrt().is_integer())
        }

        // Repeat <r1> <r3> times with initial value <r2>
        "&." => {
            if let Node::Block(_, name) = &right[0] {
                let mut loop_arg = parse_node(env.clone(), &right[1]);
                let count = parse_node(env.clone(), &right[2])
                    .literal_num()
                    .to_u32_saturating_round(rug::float::Round::Down)
                    .unwrap();
                let child_env = Rc::new(env.as_ref().clone());

                for _ in 0..count {
                    child_env.borrow_mut().define_var(name.as_ref().unwrap_or(&USCORE), loop_arg);
                    loop_arg = parse_block(child_env.clone(), &right[0]);
                }

                loop_arg
            } else {
                panic!("First argument to `&.` must be Node::Block")
            }
        }

        ":i" => todo!("Sequences needed"),

        // not <right>
        "!" => Dynamic::from(!parse_node(env.clone(), &right[0]).literal_bool()),

        "$" => todo!("Sequences needed"),

        // Floor <right>
        ":v" => {
            let right = parse_node(env.clone(), &right[0]);
            right.mutate_num(|n| n.floor())
        }

        // Ceil <right>
        ":^" => {
            let right = parse_node(env.clone(), &right[0]);
            right.mutate_num(|n| n.floor())
        }

        // Inc <right>
        "++" => {
            if let Node::Variable(name) = &right[0] {
                let mut val = env.borrow().vars.get(name).expect("Variable not recognized").clone();
                val = val.mutate_num(|n| n + 1);
                env.borrow_mut().define_var(name, val.clone());
                val
            } else {
                let right = parse_node(env.clone(), &right[0]);
                right.mutate_num(|n| n + 1)
            }
        }

        // Dec <right>
        "--" => {
            if let Node::Variable(name) = &right[0] {
                let mut val = env.borrow().vars.get(name).expect("Variable not recognized").clone();
                val = val.mutate_num(|n| n - 1);
                env.borrow_mut().define_var(name, val.clone());
                val
            } else {
                let right = parse_node(env.clone(), &right[0]);
                right.mutate_num(|n| n - 1)
            }
        }

        // <right> ^ 2
        ":*" => parse_node(env.clone(), &right[0]).mutate_num(|n| n.square()),

        // √<right>
        ":/" => parse_node(env.clone(), &right[0]).mutate_num(|n| n.sqrt()),

        // 2<right>
        ":+" => parse_node(env.clone(), &right[0]).mutate_num(|n| n * 2),

        // ½<right>
        ":-" => parse_node(env.clone(), &right[0]).mutate_num(|n| n / 2),

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
            let mut left = parse_node(env.clone(), &left[0]).literal_string();
            left.push_str(&parse_node(env.clone(), &right[0]).literal_string());
            Dynamic::from(left)
        }

        // Very weakly typed, see `src/utils/types.rs`, PartialEq for Dynamic
        // <left> == <right>
        "=" => Dynamic::from(parse_node(env.clone(), &left[0]) == parse_node(env.clone(), &right[0])),

        // <left> != <right>
        "!=" => Dynamic::from(parse_node(env.clone(), &left[0]) != parse_node(env.clone(), &right[0])),

        // <left> < <right>
        "<" => Dynamic::from(
            parse_node(env.clone(), &left[0]).literal_num() < parse_node(env.clone(), &right[0]).literal_num(),
        ),

        // <left> <= <right>
        "<=" => Dynamic::from(
            parse_node(env.clone(), &left[0]).literal_num() <= parse_node(env.clone(), &right[0]).literal_num(),
        ),

        // <left> > <right>
        ">" => Dynamic::from(
            parse_node(env.clone(), &left[0]).literal_num() > parse_node(env.clone(), &right[0]).literal_num(),
        ),

        // <left> >= <right>
        ">=" => Dynamic::from(
            parse_node(env.clone(), &left[0]).literal_num() >= parse_node(env.clone(), &right[0]).literal_num(),
        ),

        // <left> && <right> yields <right> if both truthy
        "&&" => {
            let left = parse_node(env.clone(), &left[0]);
            let right = parse_node(env.clone(), &right[0]);

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
            let left = parse_node(env.clone(), &left[0]);
            let right = parse_node(env.clone(), &right[0]);

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
            let child_env = Rc::new(env.as_ref().clone());
            if let Node::Block(_, name) = &left[0] {
                let val = child_env.borrow().vars.get("_").unwrap().clone();
                child_env.borrow_mut().define_var(
                    name.as_ref().unwrap_or(&USCORE),
                    val,
                )
            }
            let mut block = parse_block(child_env.clone(), &left[0]);

            while {
                let c_env = Rc::new(env.as_ref().clone());
                if let Node::Block(_, name) = &right[0] {
                    c_env.borrow_mut().define_var(name.as_ref().unwrap_or(&USCORE), block.clone())
                } else {
                    c_env.borrow_mut().define_var("_", block.clone());
                }

                parse_block(c_env.clone(), &right[0]).literal_bool()
            } {
                if let Node::Block(_, name) = &left[0] {
                    child_env.borrow_mut().define_var(name.as_ref().unwrap_or(&USCORE), block.clone())
                } else {
                    child_env.borrow_mut().define_var("_", block.clone());
                }

                block = parse_block(child_env.clone(), &left[0]);
            }

            block
        }

        _ => unimplemented!(),
    }
}

// Parses Node::Block, assuming it's key has already been initialized
// Call if the key is checked before
fn parse_block(env: Env, block: &Node) -> Dynamic {
    match block {
        Node::Block(block, _) => {
            for node in &block[..block.len() - 1] {
                parse_node(env.clone(), node);
            }

            parse_node(env.clone(), block.last().unwrap_or(&DEFAULT))
        }

        _ => unreachable!(),
    }
}

pub fn parse_node(env: Env, node: &Node) -> Dynamic {
    match node {
        Node::Op(op, left, right) => parse_op(env, op, left, right),

        Node::String(v) => Dynamic::from(v.clone()),

        Node::Number(v) => Dynamic::from(v.clone()),

        Node::Variable(v) => {
            if let Some(val) = env.borrow().vars.get(v) {
                val.clone()
            } else {
                panic!("Unrecognized variable {}", v);
            }
        }

        Node::Group(body) => {
            for node in &body[..body.len() - 1] {
                parse_node(env.clone(), node);
            }

            parse_node(env.clone(), body.last().unwrap_or(&DEFAULT))
        }

        Node::Block(body, name) => {
            let child_env = Rc::new(env.as_ref().clone());
            let val = child_env.borrow().vars.get("_").unwrap().clone();
            child_env.borrow_mut().define_var(
                name.as_ref().unwrap_or(&USCORE),
                val,
            );
            for node in &body[..body.len() - 1] {
                parse_node(child_env.clone(), node);
            }

            parse_node(child_env.clone(), body.last().unwrap_or(&DEFAULT))
        }
    }
}

pub fn parse(ast: &[Node]) {
    let mut env = Environment::init();

    let mut stdin = String::new();
    io::stdin()
        .read_to_string(&mut stdin)
        .expect("Could not read from stdin");

    env.define_var("_", stdin.trim_end_matches('\n').to_owned());
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

    let env: Env = Rc::new(RefCell::new(env));

    for node in &ast[..ast.len() - 1] {
        parse_node(env.clone(), node);
    }

    println!("{}", parse_node(env, ast.last().unwrap_or(&DEFAULT)));
}

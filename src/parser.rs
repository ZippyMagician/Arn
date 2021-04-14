use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;

use crate::utils::num::{to_u32, Num};
use crate::utils::{env::Environment, tokens::Node, types::*};
use crate::FLOAT_PRECISION;

lazy_static! {
    static ref DEFAULT: Node = Node::String(String::new());
    static ref USCORE: String = String::from("_");
}

pub fn parse_op(env: Env, op: &str, left: &[Node], right: &[Node]) -> Dynamic {
    match op {
        // <right>(<left>)
        "." => {
            if let Node::Variable(v) = &right[0] {
                let arg = parse_node(Rc::clone(&env), &left[0]);
                env.borrow_mut().attempt_call(v, arg)
            } else {
                panic!("Dot operator only accepts a variable on the right hand side")
            }
        }

        // <left> pow <right>
        "^" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            if left.is_string() {
                left.mutate_string(|s| s.repeat(to_u32(&env, &right[0]) as usize))
            } else {
                let mut left = left.literal_num();
                let o = left.clone();
                let right = to_u32(&env, &right[0]);
                for _ in 1..right {
                    left *= o.clone();
                }

                Dynamic::from(left)
            }
        }

        // <left> × <right>
        "*" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            left.mutate_num(|n| n * parse_node(Rc::clone(&env), &right[0]).literal_num())
        }

        // <left> ÷ <right>
        "/" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            left.mutate_num(|n| n / parse_node(Rc::clone(&env), &right[0]).literal_num())
        }

        // <left> mod <right>
        "%" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            left.mutate_num(|n| n % parse_node(Rc::clone(&env), &right[0]).literal_num())
        }

        // <left>.join(<right>)
        ":|" => {
            let mut left = parse_node(Rc::clone(&env), &left[0]).literal_array();
            left.set_env(Rc::clone(&env));
            Dynamic::from(
                left.map(|dy| format!("{}", dy))
                    .collect::<Vec<String>>()
                    .join(&parse_node(Rc::clone(&env), &right[0]).literal_string()),
            )
        }

        // <left>.split(<right>)
        ":!" => {
            let left = parse_node(Rc::clone(&env), &left[0]).literal_string();
            let right = parse_node(Rc::clone(&env), &right[0]).literal_string();

            Dynamic::from(
                left.split(&right)
                    .map(str::to_owned)
                    .collect::<Vec<String>>(),
            )
        }

        // <left> + <right>
        "+" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            left.mutate_num(|n| n + parse_node(Rc::clone(&env), &right[0]).literal_num())
        }

        // <left> - <right>
        "-" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            left.mutate_num(|n| n - parse_node(Rc::clone(&env), &right[0]).literal_num())
        }

        // <left> ==> [<left>[..<right>], <left>[<right>..]]
        ".$" => {
            let mut left = parse_node(Rc::clone(&env), &left[0]).literal_array();
            let i = to_u32(&env, &right[0]) as usize;
            left.set_env(Rc::clone(&env));
            let seq = left.collect::<Vec<Dynamic>>();

            let (l, r) = seq.split_at(i);
            Dynamic::from([l, r])
        }

        // [<left>, <right>]
        "=>" => {
            let left = to_u32(&env, &left[0]) as usize;
            let right = to_u32(&env, &right[0]) as usize;

            Dynamic::new(
                Val::Array(Box::new(Sequence::from_iter(
                    (left..=right).map(|n| Dynamic::from(Num::with_val(*FLOAT_PRECISION, n))),
                    Node::Block(vec![], None),
                    Some(right - left + 1),
                ))),
                4,
            )
        }

        // [<left>, <right>)
        "->" => {
            let left = to_u32(&env, &left[0]) as usize;
            let right = to_u32(&env, &right[0]) as usize;

            Dynamic::new(
                Val::Array(Box::new(Sequence::from_iter(
                    (left..right).map(|n| Dynamic::from(Num::with_val(*FLOAT_PRECISION, n))),
                    Node::Block(vec![], None),
                    Some(right - left),
                ))),
                4,
            )
        }

        // [1, <right>]
        "~" => {
            let right = to_u32(&env, &right[0]) as usize;

            Dynamic::new(
                Val::Array(Box::new(Sequence::from_iter(
                    (1..=right).map(|n| Dynamic::from(Num::with_val(*FLOAT_PRECISION, n))),
                    Node::Block(vec![], None),
                    Some(right),
                ))),
                4,
            )
        }

        // <left>.length
        "#" => Dynamic::from(Num::with_val(
            *FLOAT_PRECISION,
            parse_node(Rc::clone(&env), &left[0])
                .literal_array()
                .len()
                .expect("Cannot take length of infinite sequence"),
        )),

        ":_" => {
            let orig = parse_node(Rc::clone(&env), &left[0]).literal_array();
            if !orig.is_finite() {
                panic!("Cannot flatten infinite sequence");
            }

            // Get a ballpark for allocation size
            let mut new = Vec::with_capacity(
                orig.len().unwrap()
                    * orig
                        .clone()
                        .next()
                        .unwrap()
                        .literal_array()
                        .len()
                        .expect("Cannot flatten sequence of inifnite sequences"),
            );
            for dy in orig {
                if dy.is_array() {
                    for n in parse_node(Rc::clone(&env), &dy.into_node()).literal_array() {
                        new.push(n);
                    }
                } else {
                    new.push(dy.clone());
                }
            }

            Dynamic::from(new)
        }

        ".@" => todo!("Sequences needed"),

        // |<left>|
        ".|" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            left.mutate_num(Num::abs)
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
            let left = parse_node(Rc::clone(&env), &left[0]).literal_num();
            Dynamic::from(left.sqrt().is_integer())
        }

        // Repeat <r1> <r3> times with initial value <r2>
        "&." => {
            if let Node::Block(_, name) = &right[0] {
                let mut loop_arg = parse_node(Rc::clone(&env), &right[1]);
                let count = parse_node(Rc::clone(&env), &right[2])
                    .literal_num()
                    .to_u32_saturating_round(rug::float::Round::Down)
                    .unwrap();
                let child_env = Rc::new(env.as_ref().clone());

                for _ in 0..count {
                    child_env
                        .borrow_mut()
                        .define_var(name.as_ref().unwrap_or(&USCORE), loop_arg);
                    loop_arg = parse_block(Rc::clone(&child_env), &right[0]);
                }

                loop_arg
            } else {
                panic!("First argument to `&.` must be Node::Block")
            }
        }

        ":i" => todo!("Sequences needed"),

        // not <right>
        "!" => Dynamic::from(!parse_node(Rc::clone(&env), &right[0]).literal_bool()),

        "$" => todo!("Sequences needed"),

        // Floor <right>
        ":v" => {
            let right = parse_node(Rc::clone(&env), &right[0]);
            right.mutate_num(Num::floor)
        }

        // Ceil <right>
        ":^" => {
            let right = parse_node(Rc::clone(&env), &right[0]);
            right.mutate_num(Num::ceil)
        }

        // Inc <right>
        "++" => {
            if let Node::Variable(name) = &right[0] {
                let mut val = env
                    .borrow()
                    .vars
                    .get(name)
                    .expect("Variable not recognized")
                    .clone();
                val = val.mutate_num(|n| n + 1);
                env.borrow_mut().define_var(name, val.clone());
                val
            } else {
                let right = parse_node(Rc::clone(&env), &right[0]);
                right.mutate_num(|n| n + 1)
            }
        }

        // Dec <right>
        "--" => {
            if let Node::Variable(name) = &right[0] {
                let mut val = env
                    .borrow()
                    .vars
                    .get(name)
                    .expect("Variable not recognized")
                    .clone();
                val = val.mutate_num(|n| n - 1);
                env.borrow_mut().define_var(name, val.clone());
                val
            } else {
                let right = parse_node(Rc::clone(&env), &right[0]);
                right.mutate_num(|n| n - 1)
            }
        }

        // <right> ^ 2
        ":*" => parse_node(Rc::clone(&env), &right[0]).mutate_num(Num::square),

        // √<right>
        ":/" => parse_node(Rc::clone(&env), &right[0]).mutate_num(Num::sqrt),

        // 2<right>
        ":+" => parse_node(Rc::clone(&env), &right[0]).mutate_num(|n| n * 2),

        // ½<right>
        ":-" => parse_node(Rc::clone(&env), &right[0]).mutate_num(|n| n / 2),

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
            let mut left = parse_node(Rc::clone(&env), &left[0]).literal_string();
            left.push_str(&parse_node(Rc::clone(&env), &right[0]).literal_string());
            Dynamic::from(left)
        }

        // Very weakly typed, see `src/utils/types.rs`, PartialEq for Dynamic
        // <left> == <right>
        "=" => Dynamic::from(
            parse_node(Rc::clone(&env), &left[0]) == parse_node(Rc::clone(&env), &right[0]),
        ),

        // <left> != <right>
        "!=" => Dynamic::from(
            parse_node(Rc::clone(&env), &left[0]) != parse_node(Rc::clone(&env), &right[0]),
        ),

        // <left> < <right>
        "<" => Dynamic::from(
            parse_node(Rc::clone(&env), &left[0]).literal_num()
                < parse_node(Rc::clone(&env), &right[0]).literal_num(),
        ),

        // <left> <= <right>
        "<=" => Dynamic::from(
            parse_node(Rc::clone(&env), &left[0]).literal_num()
                <= parse_node(Rc::clone(&env), &right[0]).literal_num(),
        ),

        // <left> > <right>
        ">" => Dynamic::from(
            parse_node(Rc::clone(&env), &left[0]).literal_num()
                > parse_node(Rc::clone(&env), &right[0]).literal_num(),
        ),

        // <left> >= <right>
        ">=" => Dynamic::from(
            parse_node(Rc::clone(&env), &left[0]).literal_num()
                >= parse_node(Rc::clone(&env), &right[0]).literal_num(),
        ),

        // <left> && <right> yields <right> if both truthy
        "&&" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            let right = parse_node(Rc::clone(&env), &right[0]);

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
            let left = parse_node(Rc::clone(&env), &left[0]);
            let right = parse_node(Rc::clone(&env), &right[0]);

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
                child_env
                    .borrow_mut()
                    .define_var(name.as_ref().unwrap_or(&USCORE), val)
            }
            let mut block = parse_block(Rc::clone(&child_env), &left[0]);

            while {
                let child_env = Rc::new(env.as_ref().clone());
                if let Node::Block(_, name) = &right[0] {
                    child_env
                        .borrow_mut()
                        .define_var(name.as_ref().unwrap_or(&USCORE), block.clone())
                } else {
                    child_env.borrow_mut().define_var("_", block.clone());
                }

                parse_block(Rc::clone(&child_env), &right[0]).literal_bool()
            } {
                if let Node::Block(_, name) = &left[0] {
                    child_env
                        .borrow_mut()
                        .define_var(name.as_ref().unwrap_or(&USCORE), block.clone())
                } else {
                    child_env.borrow_mut().define_var("_", block.clone());
                }

                block = parse_block(Rc::clone(&child_env), &left[0]);
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
                parse_node(Rc::clone(&env), node);
            }

            parse_node(Rc::clone(&env), block.last().unwrap_or(&DEFAULT))
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
                parse_node(Rc::clone(&env), node);
            }

            parse_node(Rc::clone(&env), body.last().unwrap_or(&DEFAULT))
        }

        Node::Block(body, name) => {
            let child_env = Rc::new(env.as_ref().clone());
            let val = child_env.borrow().vars.get("_").unwrap().clone();
            child_env
                .borrow_mut()
                .define_var(name.as_ref().unwrap_or(&USCORE), val);
            for node in &body[..body.len() - 1] {
                parse_node(Rc::clone(&child_env), node);
            }

            parse_node(Rc::clone(&child_env), body.last().unwrap_or(&DEFAULT))
        }

        // This will maybe be parsed differently in the future?
        Node::Sequence(arr, block, len) => Dynamic::new(
            Val::Array(Box::new(Sequence::from_vec_dyn(
                &arr.iter()
                    .map(|n| parse_node(Rc::clone(&env), n))
                    .collect::<Vec<Dynamic>>(),
                block.as_ref().clone(),
                *len,
            ))),
            4,
        ),
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
        parse_node(Rc::clone(&env), node);
    }

    println!("{}", parse_node(env, ast.last().unwrap_or(&DEFAULT)));
}

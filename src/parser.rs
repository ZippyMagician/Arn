use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;

use rand::Rng;

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

        ".@" => {
            let mut parent = parse_node(Rc::clone(&env), &left[0]).literal_array();
            parent.set_env(Rc::clone(&env));

            let mut pre = Vec::new();
            for item in parent {
                pre.push(item.literal_array());
            }

            Dynamic::from(
                pre.clone()
                    .get(0)
                    .unwrap()
                    .clone()
                    .enumerate()
                    .map(|(i, _)| {
                        pre.iter()
                            .map(|array| array.clone().nth(i).unwrap())
                            .collect()
                    })
                    .collect::<Vec<Vec<Dynamic>>>(),
            )
        }

        // |<left>|
        ".|" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            left.mutate_num(Num::abs)
        }

        // Reverse <left>
        ".<" => Dynamic::from(
            parse_node(Rc::clone(&env), &left[0])
                .literal_array()
                .rev()
                .collect::<Vec<Dynamic>>(),
        ),

        // Rangify <left>, exclusive or inclusive
        ".." | ".=" => {
            let mut left = parse_node(Rc::clone(&env), &left[0]).literal_array();
            left.set_env(Rc::clone(&env));
            let left: Vec<u32> = left
                .map(|n| {
                    n.literal_num()
                        .to_u32_saturating_round(rug::float::Round::Down)
                        .unwrap()
                })
                .collect();

            #[allow(clippy::range_plus_one)]
            let range = if op == ".." {
                left[0]..left[1]
            } else {
                left[0]..left[1] + 1
            };

            Dynamic::new(
                Val::Array(Box::new(Sequence::from_iter(
                    range
                        .clone()
                        .map(|n| Dynamic::from(Num::with_val(*FLOAT_PRECISION, n))),
                    Node::Block(vec![], None),
                    Some(range.count()),
                ))),
                4,
            )
        }

        // Split <left> on newlines
        ":n" => Dynamic::from(
            parse_node(Rc::clone(&env), &left[0])
                .literal_string()
                .split('\n')
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        ),

        // Split <left> on spaces
        ":s" => Dynamic::from(
            parse_node(Rc::clone(&env), &left[0])
                .literal_string()
                .split(' ')
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        ),

        // Tail of <left>
        ":}" => {
            let mut seq = parse_node(Rc::clone(&env), &left[0]).literal_array();
            seq.set_env(Rc::clone(&env));

            seq.last()
                .expect("Cannot take last element of infinite sequence")
        }

        // Head of <left>
        ":{" => {
            let mut seq = parse_node(Rc::clone(&env), &left[0]).literal_array();
            seq.set_env(Rc::clone(&env));

            seq.next().unwrap()
        }

        // Behead <left>
        ".{" => {
            let mut seq = parse_node(Rc::clone(&env), &left[0]).literal_array();
            seq.set_env(Rc::clone(&env));

            Dynamic::from(seq.collect::<Vec<Dynamic>>()[1..].to_owned())
        }

        // Drop <left>
        ".}" => {
            let mut seq = parse_node(Rc::clone(&env), &left[0]).literal_array();
            seq.set_env(Rc::clone(&env));

            Dynamic::from(seq.clone().collect::<Vec<Dynamic>>()[..seq.count() - 1].to_owned())
        }

        // Group entries in <left> based on frequencies
        ":@" => {
            let mut arr = parse_node(Rc::clone(&env), &left[0]).literal_array();
            arr.set_env(Rc::clone(&env));
            let arr = arr.collect::<Vec<Dynamic>>();

            Dynamic::from(arr.iter().fold(Vec::new(), |mut acc, val| {
                if acc.is_empty() {
                    vec![vec![val.clone()]]
                } else {
                    let filter = acc.iter().filter(|e| e[0].clone() == val.clone());
                    if filter.clone().count() > 0 {
                        let filter = filter.cloned().collect::<Vec<_>>();
                        let pos = acc.iter().cloned().position(|e| e == filter[0]).unwrap();
                        acc[pos].push(val.clone());
                        acc
                    } else {
                        acc.push(vec![val.clone()]);
                        acc
                    }
                }
            }))
        }

        // is <left> perfect square?
        "^*" => {
            let left = parse_node(Rc::clone(&env), &left[0]).literal_num();
            Dynamic::from(left.sqrt().is_integer())
        }

        // Repeat <r1> <r3> times with initial value <r2>
        "&." => {
            let mut loop_arg = parse_node(Rc::clone(&env), &right[1]);
            let count = parse_node(Rc::clone(&env), &right[2])
                .literal_num()
                .to_u32_saturating_round(rug::float::Round::Down)
                .unwrap();
            let child_env = Rc::new(env.as_ref().clone());

            for _ in 0..count {
                if let Node::Block(_, name) = &right[0] {
                    child_env
                        .borrow_mut()
                        .define_var(name.as_ref().unwrap_or(&USCORE), loop_arg)
                } else {
                    child_env.borrow_mut().define_var("_", loop_arg);
                }
                loop_arg = parse_node_uniq(Rc::clone(&child_env), &right[0]);
            }

            loop_arg
        }

        ":i" => {
            let mut seq = parse_node(Rc::clone(&env), &left[0]).literal_array();
            let val = parse_node(Rc::clone(&env), &right[0]);
            seq.set_env(Rc::clone(&env));

            Dynamic::from(Num::with_val(
                *FLOAT_PRECISION,
                seq.position(|e| e.clone() == val)
                    .map(|v| v as i128)
                    .unwrap_or(-1),
            ))
        }

        // not <right>
        "!" => Dynamic::from(!parse_node(Rc::clone(&env), &right[0]).literal_bool()),

        // Filter <r2> with condition <r1> /// <r2>.any(<r1>)
        "$" | "$:" => {
            let mut seq = parse_node(Rc::clone(&env), &right[1]).literal_array();
            seq.set_env(Rc::clone(&env));
            let child_env = Rc::new(env.as_ref().clone());
            let filter = seq
                .filter(|v| {
                    if let Node::Block(_, name) = &right[0] {
                        child_env
                            .borrow_mut()
                            .define_var(name.as_ref().unwrap_or(&USCORE), v.clone())
                    } else {
                        child_env.borrow_mut().define_var("_", v.clone());
                    }

                    parse_node_uniq(Rc::clone(&child_env), &right[0]).literal_bool()
                })
                .collect::<Vec<_>>();

            if op == "$" {
                Dynamic::from(filter)
            } else {
                Dynamic::from(filter.len() > 0)
            }
        }

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

        // Sort <right> in descending order
        ":>" => {
            let mut seq = parse_node(Rc::clone(&env), &right[0]).literal_array();
            seq.set_env(Rc::clone(&env));
            let mut seq = seq.collect::<Vec<_>>();
            seq.sort_by(|a, b| b.partial_cmp(a).unwrap());
            Dynamic::from(seq)
        }

        // Sort <right> in ascending order
        ":<" => {
            let mut seq = parse_node(Rc::clone(&env), &right[0]).literal_array();
            seq.set_env(Rc::clone(&env));
            let mut seq = seq.collect::<Vec<_>>();
            seq.sort_by(|a, b| a.partial_cmp(b).unwrap());
            Dynamic::from(seq)
        }

        // Bifurcate <right>
        "|:" => {
            let string = parse_node(Rc::clone(&env), &right[0]).literal_string();
            let mid = string.len() / 2;
            Dynamic::from([
                string[..mid].to_owned(),
                string[mid..].to_owned().chars().rev().collect(),
            ])
        }

        // Get random item within <right>
        "?." => {
            let mut seq = parse_node(Rc::clone(&env), &right[0]).literal_array();
            seq.set_env(Rc::clone(&env));
            let seq = seq.collect::<Vec<_>>();

            let mut rng = rand::thread_rng();

            Dynamic::from(seq[(rng.gen::<f64>() * seq.len() as f64).floor() as usize].clone())
        }

        // All primes up to <right>
        "#." => {
            let num = to_u32(&env, &right[0]) as usize;
            let sieve = primal::Primes::all()
                .take_while(|n| *n <= num)
                .map(|n| Num::with_val(*FLOAT_PRECISION, n))
                .collect::<Vec<_>>();
            Dynamic::new(
                Val::Array(Box::new(Sequence::from_vec(
                    &sieve,
                    Node::Block(vec![], None),
                    Some(sieve.len()),
                ))),
                4,
            )
        }

        // All factors of <right>
        "*." => {
            let num = to_u32(&env, &right[0]) as usize;
            let mut fac = Vec::new();
            let mut i = 1;
            let mut ind = 0;

            while i <= (num as f64).sqrt().floor() as usize {
                if num % i == 0 {
                    fac.insert(ind, Num::with_val(*FLOAT_PRECISION, i));
                    if i != num / i {
                        fac.insert(fac.len() - ind, Num::with_val(*FLOAT_PRECISION, num / i));
                    }
                    ind += 1;
                }

                i += 1;
            }

            let len = fac.len();
            let temp = fac[len - 1].clone();
            fac[len - 1] = fac[0].clone();
            fac[0] = temp;

            Dynamic::from(fac)
        }

        // Split array at mid
        "$." => {
            let mut seq = parse_node(Rc::clone(&env), &right[0]).literal_array();
            seq.set_env(Rc::clone(&env));
            let seq = seq.collect::<Vec<_>>();

            Dynamic::from([
                seq[..seq.len() / 2].to_owned(),
                seq[seq.len() / 2..].to_owned(),
            ])
        }

        // Zip <left> and <right>
        "z" => {
            let left = parse_node(Rc::clone(&env), &left[0])
                .literal_array()
                .set_env_self(Rc::clone(&env))
                .collect::<Vec<_>>();
            let right = parse_node(Rc::clone(&env), &right[0])
                .literal_array()
                .set_env_self(Rc::clone(&env))
                .collect::<Vec<_>>();
            let mut output = Vec::with_capacity(left.len());

            for i in 0..left.len() {
                output.push([left[i].clone(), right[i].clone()]);
            }

            Dynamic::from(output)
        }

        // Concat <left> and <right>, special case for arrays
        "|" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            let right = parse_node(Rc::clone(&env), &right[0]);
            if left.is_array() {
                let mut left = left
                    .literal_array()
                    .set_env_self(Rc::clone(&env))
                    .collect::<Vec<_>>();
                if right.is_array() {
                    Dynamic::from(
                        [
                            left,
                            right
                                .literal_array()
                                .set_env_self(Rc::clone(&env))
                                .collect::<Vec<_>>(),
                        ]
                        .concat()
                        .to_owned(),
                    )
                } else {
                    left.push(right);
                    Dynamic::from(left)
                }
            } else if right.is_array() {
                let mut right = right
                    .literal_array()
                    .set_env_self(Rc::clone(&env))
                    .collect::<Vec<_>>();
                right.insert(0, left);
                Dynamic::from(right)
            } else {
                let mut left = left.literal_string();
                left.push_str(&right.literal_string());
                Dynamic::from(left)
            }
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
            let mut block = parse_node_uniq(Rc::clone(&child_env), &left[0]);

            while {
                let child_env = Rc::new(env.as_ref().clone());
                if let Node::Block(_, name) = &right[0] {
                    child_env
                        .borrow_mut()
                        .define_var(name.as_ref().unwrap_or(&USCORE), block.clone())
                } else {
                    child_env.borrow_mut().define_var("_", block.clone());
                }

                parse_node_uniq(Rc::clone(&child_env), &right[0]).literal_bool()
            } {
                if let Node::Block(_, name) = &left[0] {
                    child_env
                        .borrow_mut()
                        .define_var(name.as_ref().unwrap_or(&USCORE), block.clone())
                } else {
                    child_env.borrow_mut().define_var("_", block.clone());
                }

                block = parse_node_uniq(Rc::clone(&child_env), &left[0]);
            }

            block
        }

        _ => unimplemented!(),
    }
}

// Parses Node::Block, assuming it's key has already been initialized
// Call if the key is checked before
fn parse_node_uniq(env: Env, block: &Node) -> Dynamic {
    match block {
        Node::Block(block, _) => {
            for node in &block[..block.len() - 1] {
                parse_node(Rc::clone(&env), node);
            }

            parse_node(Rc::clone(&env), block.last().unwrap_or(&DEFAULT))
        }

        _ => parse_node(env, block),
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
        Node::Sequence(arr, block, len) => {
            let mut seq = Sequence::from_vec_dyn(
                &arr.iter()
                    .map(|n| parse_node(Rc::clone(&env), n))
                    .collect::<Vec<Dynamic>>(),
                block.as_ref().clone(),
                len.as_ref().map(|n| to_u32(&env, n.as_ref()) as usize),
            );

            seq.set_env(Rc::clone(&env));
            Dynamic::new(Val::Array(Box::new(seq)), 4)
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
        parse_node(Rc::clone(&env), node);
    }

    println!("{}", parse_node(env, ast.last().unwrap_or(&DEFAULT)));
}

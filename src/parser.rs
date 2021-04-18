use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;

use radix_fmt::radix;
use rand::Rng;

use crate::utils::num::{to_u32, Num};
use crate::utils::{self, env::Environment, tokens::Node, types::*};
use crate::{FLOAT_PRECISION, MATCHES};

lazy_static! {
    static ref DEFAULT: Node = Node::String(String::new());
    static ref USCORE: String = String::from("_");
}

fn grab_block_from_fold(fold: &Node, mut block: Option<Node>) -> (Option<Node>, Node) {
    match fold {
        Node::Op(n, l, r) => {
            let mut r = r.clone();
            let end = r.len() - 1;
            let inter = grab_block_from_fold(&r[end], block);

            block = inter.0;
            r[end] = inter.1;
            (block, Node::Op(n.clone(), l.clone(), r))
        }

        Node::Block(_, _) => (Some(fold.clone()), Node::Variable("_".to_string())),

        _ => (block, fold.clone()),
    }
}

pub fn parse_op(env: Env, op: &str, left: &[Node], right: &[Node]) -> Dynamic {
    match op {
        // <right>(<left>)
        "." => {
            if let Node::Variable(v) = &right[0] {
                let arg = parse_node(Rc::clone(&env), &left[0]);
                env.borrow().attempt_call(v, &env, arg)
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
            let right = parse_node(Rc::clone(&env), &right[0]).literal_num();
            left.mutate_num(|n| n * right)
        }

        // <left> ÷ <right>
        "/" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            let right = parse_node(Rc::clone(&env), &right[0]).literal_num();
            left.mutate_num(|n| n / right)
        }

        // <left> mod <right>
        "%" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            let right = parse_node(Rc::clone(&env), &right[0]).literal_num();
            left.mutate_num(|n| n % right)
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
            let right = parse_node(Rc::clone(&env), &right[0]).literal_num();
            left.mutate_num(|n| n + right)
        }

        // <left> - <right>
        "-" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            let right = parse_node(Rc::clone(&env), &right[0]).literal_num();
            left.mutate_num(|n| n - right)
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

        ";" => {
            let ops = format!("{}", right[0]);
            let chars = ops.trim().trim_matches('"').chars();
            let mut cur = parse_node(Rc::clone(&env), &left[0]).to_string();

            for char in chars {
                cur =
                    match char {
                        'b' => radix(cur.parse::<i128>().expect("Invalid base10 number"), 2)
                            .to_string(),
                        'o' => radix(cur.parse::<i128>().expect("Invalid base10 number"), 8)
                            .to_string(),
                        'h' => radix(cur.parse::<i128>().expect("Invalid base10 number"), 16)
                            .to_string(),
                        'B' => i128::from_str_radix(&cur, 2)
                            .expect("Invalid base2 number")
                            .to_string(),
                        'O' => i128::from_str_radix(&cur, 8)
                            .expect("Invalid base8 number")
                            .to_string(),
                        'H' => i128::from_str_radix(&cur, 16)
                            .expect("Invalid base16 number")
                            .to_string(),
                        _ => panic!("Unrecognized base conversion char {}", char),
                    }
            }

            Dynamic::from(cur)
        }

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
                seq.position(|e| e == val).map_or(-1, |v| v as i128),
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
                Dynamic::from(!filter.is_empty())
            }
        }

        // Fold + map op
        "\\" => {
            let seq = Box::new(parse_node(Rc::clone(&env), &right[0])
                .literal_array()
                .set_env_self(Rc::clone(&env)));

            let (block, rest) = grab_block_from_fold(&left[0], None);

            let res = Box::new(if let Some(Node::Block(_, name)) = block.clone() {
                let child_env = Rc::new(env.as_ref().clone());
                let block = block.unwrap();

                seq.map(|val| {
                    child_env
                        .borrow_mut()
                        .define_var(name.as_ref().unwrap_or(&USCORE), val);
                    parse_node_uniq(Rc::clone(&child_env), &block)
                })
                .collect::<Vec<_>>()
            } else {
                seq.collect::<Vec<_>>()
            });

            if rest == Node::Variable("_".to_string()) {
                Dynamic::from(res.as_ref().clone())
            } else {
                let constructed = format!("{}", rest);
                let seperator = constructed.trim().trim_matches('_');
                let program = res
                    .iter()
                    .map(|n| format!("{}", n))
                    .collect::<Vec<String>>()
                    .join(seperator);

                // This looks like a Vec<Node>, but in reality it is a single Node (only one value)
                let val = crate::build_ast(&program);
                parse_node(Rc::clone(&env), &val[0])
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

            seq[(rng.gen::<f64>() * seq.len() as f64).floor() as usize].clone()
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
                        .concat(),
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

        "@" => {
            let seq = parse_node(Rc::clone(&env), &left[0])
                .literal_array()
                .set_env_self(Rc::clone(&env));
            let child_env = Rc::new(env.as_ref().clone());

            Dynamic::from(
                seq.map(|val| {
                    if let Node::Block(_, name) = &left[0] {
                        child_env
                            .borrow_mut()
                            .define_var(name.as_ref().unwrap_or(&USCORE), val)
                    } else {
                        child_env.borrow_mut().define_var("_", val);
                    }

                    parse_node_uniq(Rc::clone(&child_env), &right[0])
                })
                .collect::<Vec<_>>(),
            )
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

    let mut stdin = if atty::is(atty::Stream::Stdin) {
        String::from("")
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Could not read from stdin");
        buffer
    };

    if MATCHES.is_present("one-ten") {
        stdin = utils::create_str_range(1, 10);
    }
    if MATCHES.is_present("one-hundred") {
        stdin = utils::create_str_range(1, 100);
    }
    if MATCHES.is_present("rangeify") {
        stdin = utils::create_str_range(
            1,
            stdin
                .parse::<usize>()
                .expect("Input was not a valid integer"),
        );
    }

    env.define_var("_", stdin.trim_end_matches('\n').to_owned());
    env.define_var(
        "E",
        Num::with_val(
            *FLOAT_PRECISION,
            Num::parse("2.7182818284590452353602874713527").unwrap(),
        ),
    );

    env.define_fn("out", |_, d| {
        println!("{}", d);
        d
    });
    env.define_fn("f", |e, val| {
        let child = Rc::new(e.as_ref().clone());
        child.borrow_mut().define_var("_", val);
        parse_node(Rc::clone(&child), &crate::build_ast("*\\~")[0])
    });

    let env: Env = Rc::new(RefCell::new(env));

    for node in &ast[..ast.len() - 1] {
        parse_node(Rc::clone(&env), node);
    }

    let mut result = parse_node(Rc::clone(&env), ast.last().unwrap_or(&DEFAULT));

    if MATCHES.is_present("first") {
        result = result
            .literal_array()
            .set_env_self(Rc::clone(&env))
            .collect::<Vec<_>>()
            .first()
            .unwrap()
            .clone();
    }
    if MATCHES.is_present("last") {
        result = result
            .literal_array()
            .set_env_self(Rc::clone(&env))
            .collect::<Vec<_>>()
            .last()
            .unwrap()
            .clone();
    }
    if MATCHES.is_present("sum") {
        result = Dynamic::from(
            result
                .literal_array()
                .set_env_self(Rc::clone(&env))
                .map(|n| n.literal_num())
                .fold(Num::new(*FLOAT_PRECISION), |acc, val| acc + val),
        );
    }
    if MATCHES.is_present("size") {
        result = Dynamic::from(Num::with_val(
            *FLOAT_PRECISION,
            result.literal_array().set_env_self(Rc::clone(&env)).count(),
        ));
    }

    println!("{}", result);
}

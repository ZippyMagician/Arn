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
        // Assign expression <right> to <left>
        ":=" => {
            let name = format!("{}", left[0]);
            let right = right[0].clone();
            env.borrow_mut().define([name.trim()], move |env, arg| {
                let child = Rc::new(env.as_ref().clone());
                child.borrow_mut().define_var("_", arg);
                parse_node(Rc::clone(&child), &right)
            });
            Dynamic::from(false)
        }

        // <right>(<left>)
        "." => {
            let v = format!("{}", right[0]);
            let arg = parse_node(Rc::clone(&env), &left[0]);
            env.borrow().attempt_call(v.trim(), &env, arg)
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

        // [<left>, <right>]
        "<>" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            let right = parse_node(Rc::clone(&env), &right[0]);
            Dynamic::from([left, right])
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

        // Descending range [<left>, 1]
        ".~" => {
            let end = to_u32(&env, &left[0]) as usize;

            Dynamic::from(
                (1..=end)
                    .rev()
                    .map(|n| Dynamic::from(Num::with_val(*FLOAT_PRECISION, n)))
                    .collect::<Vec<_>>(),
            )
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

        // Base conversion of <left> based on <right>
        ";" => {
            let ops = if env.borrow().vals.get(&right[0].to_string()).is_none() {
                right[0].to_string()
            } else {
                parse_node(Rc::clone(&env), &right[0]).to_string()
            };
            let chars = ops.trim().trim_matches('"').chars();
            let mut cur = parse_node(Rc::clone(&env), &left[0]);
            cur = if cur.clone().to_string().matches('\n').count() > 0 {
                let temp = cur
                    .to_string()
                    .trim()
                    .split('\n')
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                if chars
                    .clone()
                    .next()
                    .map_or(true, |c| c != 'H' && c != 'O' && c != 'B')
                {
                    Dynamic::from(temp)
                } else {
                    Dynamic::from(temp.join(""))
                }
            } else {
                cur.into_string()
            };
            let mut num = String::new();

            for char in chars.chain("\u{2192}".chars()) {
                if !num.is_empty() && !char.is_numeric() {
                    cur = Dynamic::from(utils::nbase_padded(cur, |cur| {
                        radix(
                            cur.parse::<i128>().expect("Invalid base10 number"),
                            num.parse().unwrap(),
                        )
                        .to_string()
                    }));
                    num.clear();
                }

                cur = match char {
                    'b' => Dynamic::from(utils::nbase_padded(cur, |cur| {
                        radix(cur.parse::<i128>().expect("Invalid base10 number"), 2).to_string()
                    })),
                    'o' => Dynamic::from(utils::nbase_padded(cur, |cur| {
                        radix(cur.parse::<i128>().expect("Invalid base10 number"), 8).to_string()
                    })),
                    'h' => Dynamic::from(utils::nbase_padded(cur, |cur| {
                        radix(cur.parse::<i128>().expect("Invalid base10 number"), 16).to_string()
                    })),
                    'B' => Dynamic::from(
                        i128::from_str_radix(&cur.literal_string(), 2)
                            .expect("Invalid base2 number")
                            .to_string(),
                    ),
                    'O' => Dynamic::from(
                        i128::from_str_radix(&cur.literal_string(), 8)
                            .expect("Invalid base8 number")
                            .to_string(),
                    ),
                    'H' => Dynamic::from(
                        i128::from_str_radix(&cur.literal_string(), 16)
                            .expect("Invalid base16 number")
                            .to_string(),
                    ),
                    '0'..='9' => {
                        num.push(char);
                        cur
                    }
                    '\u{2192}' => cur,
                    _ => panic!("Unrecognized base conversion char {}", char),
                }
            }

            Dynamic::from(cur)
        }

        // Flatten <left>
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

        // Transpose <left>
        ":%" => {
            let mut parent = parse_node(Rc::clone(&env), &left[0]).literal_array();
            parent.set_env(Rc::clone(&env));

            let mut pre = Vec::new();
            for item in parent {
                pre.push(item.literal_array());
            }

            Dynamic::from(
                vec![
                    0;
                    pre.iter()
                        .cloned()
                        .max_by(|l, r| l.clone().count().cmp(&r.clone().count()))
                        .unwrap_or_else(|| Sequence::from_vec_dyn(
                            &[],
                            Node::String(String::new()),
                            Some(0)
                        ))
                        .count()
                ]
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    pre.iter()
                        .filter_map(|array| array.clone().nth(i))
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

        // Index of <right> in <left>
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

        // Filter <r2> with condition <r1>, <r2>.any(<r1>)
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
            let seq = Box::new(
                parse_node(Rc::clone(&env), &right[0])
                    .literal_array()
                    .set_env_self(Rc::clone(&env)),
            );

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
                    .cloned()
                    .map(|n| format!("{}", n.into_node()))
                    .collect::<Vec<_>>()
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
                let mut val = env.borrow().get_var(name);
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
                let mut val = env.borrow().get_var(name);
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

        // Dedup <right>
        "#>" => {
            let mut hash = std::collections::HashSet::new();
            let mut result = Vec::new();
            let array = parse_node(Rc::clone(&env), &right[0])
                .literal_array()
                .set_env_self(Rc::clone(&env));
            for item in array {
                if hash.get(&item).is_none() {
                    hash.insert(item.clone());
                    result.push(item);
                }
            }

            Dynamic::from(result)
        }

        // Dedup sieve
        "#:" => {
            let mut hash = std::collections::HashSet::new();
            let mut result = Vec::new();
            let array = parse_node(Rc::clone(&env), &right[0])
                .literal_array()
                .set_env_self(Rc::clone(&env));
            for item in array {
                if hash.get(&item).is_none() {
                    result.push(Num::with_val(*FLOAT_PRECISION, 1));
                    hash.insert(item);
                } else {
                    result.push(Num::new(*FLOAT_PRECISION));
                }
            }

            Dynamic::from(result)
        }

        // <left>.nth(<right>)
        "?" => {
            let mut left = parse_node(Rc::clone(&env), &left[0])
                .literal_array()
                .set_env_self(Rc::clone(&env));
            let index = to_u32(&env, &right[0]) as usize;
            left.nth(index).unwrap()
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
        ":" => {
            let child_env = Rc::new(env.as_ref().clone());
            if let Node::Block(_, name) = &left[0] {
                let val = child_env.borrow().get_var("_");
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

        // Compare adjacent values in array <left> and, if <right> evaluates to true, groups them
        "::" => {
            let mut groups = Vec::new();
            let orig = parse_node(Rc::clone(&env), &left[0])
                .literal_array()
                .set_env_self(Rc::clone(&env));

            for node in orig {
                if groups.last().is_none() {
                    groups.push(vec![node]);
                } else {
                    let mut vals = vec![
                        groups.last().unwrap().last().unwrap().clone().into_node(),
                        node.clone().into_node(),
                    ];
                    let block = utils::traverse_replace(&mut vals, right[0].clone());
                    if parse_node(Rc::clone(&env), &block).literal_bool() {
                        groups.last_mut().unwrap().push(node);
                    } else {
                        groups.push(vec![node]);
                    }
                }
            }

            Dynamic::from(groups)
        }

        // If <r1> then bind <r2> to <left>, else yield <left>
        "??" => {
            let val = parse_node(Rc::clone(&env), &left[0]);
            let condition = parse_node(Rc::clone(&env), &right[0]).literal_bool();
            let child_env = Rc::new(env.as_ref().clone());

            if condition {
                if let Node::Block(_, name) = &right[1] {
                    child_env
                        .borrow_mut()
                        .define_var(name.as_ref().unwrap_or(&USCORE), val.clone())
                } else {
                    child_env.borrow_mut().define_var("_", val.clone());
                }

                parse_node(Rc::clone(&child_env), &right[1])
            } else {
                val
            }
        }

        // Bind <right> to each value in <left>
        "@" => {
            let seq = parse_node(Rc::clone(&env), &left[0])
                .literal_array()
                .set_env_self(Rc::clone(&env));
            let child_env = Rc::new(env.as_ref().clone());

            Dynamic::from(
                seq.map(|val| {
                    if let Node::Block(_, name) = &right[0] {
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

        // Bind <right> to <left>
        "&" => {
            let left = parse_node(Rc::clone(&env), &left[0]);
            let child_env = Rc::new(env.as_ref().clone());

            if let Node::Block(_, name) = &right[0] {
                child_env
                    .borrow_mut()
                    .define_var(name.as_ref().unwrap_or(&USCORE), left)
            } else {
                child_env.borrow_mut().define_var("_", left);
            }

            parse_node_uniq(Rc::clone(&child_env), &right[0])
        }

        // Count of entries in <r2> that, when bound by <r1>, yield a truthy value
        "/:" => {
            let array = parse_node(Rc::clone(&env), &right[1])
                .literal_array()
                .set_env_self(Rc::clone(&env));
            let child_env = Rc::new(env.as_ref().clone());

            Dynamic::from(Num::with_val(
                *FLOAT_PRECISION,
                array
                    .filter(|val| {
                        if let Node::Block(_, name) = &right[0] {
                            child_env
                                .borrow_mut()
                                .define_var(name.as_ref().unwrap_or(&USCORE), val.clone())
                        } else {
                            child_env.borrow_mut().define_var("_", val.clone());
                        }

                        parse_node_uniq(Rc::clone(&child_env), &right[0]).literal_bool()
                    })
                    .count(),
            ))
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

        Node::CmpString(v, chr) => Dynamic::from(utils::dict::decompress(v, *chr == '\'')),

        Node::Number(v) => Dynamic::from(v.clone()),

        Node::Variable(v) => env
            .borrow()
            .attempt_call(v, &env, env.borrow().get_var("_")),

        Node::Group(body) => {
            for node in &body[..body.len() - 1] {
                parse_node(Rc::clone(&env), node);
            }

            parse_node(Rc::clone(&env), body.last().unwrap_or(&DEFAULT))
        }

        Node::Block(body, name) => {
            let child_env = Rc::new(env.as_ref().clone());
            let val = child_env.borrow().get_var("_");
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

macro_rules! def_builtins {
    ($env:ident; $($($name:literal),*: $value:literal);*) => {
        $(
            $env.define([$($name),*], |e, val| {
                let child = Rc::new(e.as_ref().clone());
                child.borrow_mut().define_var("_", val);
                parse_node(Rc::clone(&child), &crate::build_ast($value)[0])
            });
        )*
    }
}

pub fn parse(ast: &[Node]) {
    let mut env = Environment::init();

    let mut stdin = if let Some(val) = MATCHES.value_of("input") {
        val.to_owned()
    } else if atty::is(atty::Stream::Stdin) {
        String::from("")
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Could not read from stdin");
        buffer.trim_end_matches('\n').to_owned()
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
    if MATCHES.is_present("0-range") {
        stdin = utils::create_str_range(
            0,
            stdin
                .parse::<usize>()
                .expect("Input was not a valid integer")
                - 1,
        );
    }

    // Defined variables
    env.define_var(
        "_",
        // Eval code as input if `-e` present
        if MATCHES.is_present("eval") {
            parse_node(
                Rc::new(RefCell::new(env.clone())),
                &crate::build_ast(&stdin)[0],
            )
        } else {
            Dynamic::from(stdin.clone())
        },
    );
    env.define_var(
        "E",
        Num::with_val(
            *FLOAT_PRECISION,
            Num::parse("2.7182818284590452353602874713527").unwrap(),
        ),
    );
    env.define_var(
        "pi",
        Num::with_val(*FLOAT_PRECISION, rug::float::Constant::Pi),
    );
    env.define_var("a", Vec::<Dynamic>::new());
    env.define_var("c", String::new());
    env.define_var("Fi", "Fizz".to_string());
    env.define_var("Bu", "Buzz".to_string());
    // I don't care what people say, I am never adding a constant for "Hello, World!"

    // Defined functions
    env.define(["ol", "outl"], |_, d| {
        println!("{}", d);
        d
    });
    env.define(["o", "out"], |_, d| {
        print!("{}", d);
        d
    });
    def_builtins! {env;
        "f", "fact":        r#"*\(~||[1])"#;
        "me", "mean":       r#"(+\)/(#"#;
        "ma", "max":        r#"(:>) :{"#;
        "mo", "mode":       r#":@&ma:{"#;
        "mi", "min":        r#"(:<) :{"#;
        "med", "median":    r#"(:-#&%2=0)&&:-(((:<)?(--:-#))+((:<)?:-#))||(:<)?:v:-#"#;
        "sdev":             r#":/((@v{:*(v-me)).me"#;
        "crt", "cartesian": r#":{@a{:}@a<>}&:_"#;
        "eq", "equal":      r#":@#=1"#
    };

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
    if MATCHES.is_present("index") {
        result = result
            .literal_array()
            .set_env_self(Rc::clone(&env))
            .nth(stdin.parse::<usize>().unwrap())
            .unwrap();
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
    if MATCHES.is_present("not") {
        result = Dynamic::from(!result.literal_bool())
    }

    println!("{}", result);
}

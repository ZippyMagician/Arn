#![allow(dead_code)]

use std::fmt::{self, Display, Formatter};

use super::env::Environment;
use super::num::Num;
use super::tokens::Node;
use crate::{FLOAT_PRECISION, OUTPUT_PRECISION};

// Inner value enum for dynamic type
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    String(String),

    Number(Num),

    Boolean(bool),

    Empty,
    // TODO: Sequence type
}

// Struct that represents types in Arn
#[derive(Clone, Debug)]
#[allow(clippy::clippy::wrong_self_convention)]
pub struct Dynamic {
    val: Val,
    cur: u8,
}

impl Dynamic {
    pub fn empty() -> Self {
        Self {
            val: Val::Empty,
            cur: 0,
        }
    }

    #[inline]
    pub fn as_string(&mut self) {
        *self = self.into_string();
    }

    #[inline]
    pub fn as_num(&mut self) {
        *self = self.into_num();
    }

    #[inline]
    pub fn as_bool(&mut self) {
        *self = self.into_bool();
    }

    // Cast inner value to a `Val::String`
    pub fn into_string(&self) -> Self {
        match &self.val {
            Val::String(_) => self.clone(),

            Val::Number(n) => Self {
                val: Val::String(n.to_string()),
                cur: 1,
            },

            Val::Boolean(b) => Self {
                val: Val::String(b.to_string()),
                cur: 1,
            },

            Val::Empty => Self {
                val: Val::String(Default::default()),
                cur: 1,
            },
        }
    }

    // Cast to `Val::Number`
    pub fn into_num(&self) -> Self {
        match &self.val {
            Val::String(s) => Self {
                val: Val::Number(Num::with_val(
                    *FLOAT_PRECISION,
                    Num::parse(s).unwrap_or_else(|_| Num::parse("0").unwrap()),
                )),
                cur: 2,
            },

            Val::Number(_) => self.clone(),

            Val::Boolean(b) => Self {
                val: Val::Number(Num::with_val(*FLOAT_PRECISION, if *b { 1 } else { 0 })),
                cur: 2,
            },

            Val::Empty => Self {
                val: Val::Number(Num::with_val(*FLOAT_PRECISION, 0)),
                cur: 2,
            },
        }
    }

    // Cast to `Val::Boolean`
    pub fn into_bool(&self) -> Self {
        match &self.val {
            Val::String(s) => Self {
                val: Val::Boolean(!s.is_empty()),
                cur: 3,
            },

            Val::Number(n) => Self {
                val: Val::Boolean(*n != 0),
                cur: 3,
            },

            Val::Boolean(_) => self.clone(),

            Val::Empty => Self {
                val: Val::Boolean(false),
                cur: 3,
            },
        }
    }

    #[inline]
    pub fn literal_num(self) -> Num {
        match self.val {
            Val::Number(n) => n,
            _ => self.into_num().literal_num(),
        }
    }

    #[inline]
    pub fn literal_string(self) -> String {
        match self.val {
            Val::String(s) => s,
            _ => self.into_string().literal_string(),
        }
    }

    #[inline]
    pub fn literal_bool(self) -> bool {
        match self.val {
            Val::Boolean(b) => b,
            _ => self.into_bool().literal_bool(),
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        self.cur == 1
    }

    #[inline]
    pub fn is_num(&self) -> bool {
        self.cur == 2
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        self.cur == 3
    }

    // Mutate inner `Val::String`
    pub fn mutate_string<T: FnOnce(String) -> String>(&self, f: T) -> Self {
        match &self.val {
            Val::String(s) => Self::from(f(s.clone())),

            _ => self.into_string().mutate_string(f),
        }
    }

    // Mutate inner `Val::Number`
    pub fn mutate_num<T: FnOnce(Num) -> Num>(&self, f: T) -> Self {
        match &self.val {
            Val::Number(n) => Self::from(f(n.clone())),

            _ => self.into_num().mutate_num(f),
        }
    }

    // Mutate inner `Val::Boolean`
    pub fn mutate_bool<T: FnOnce(bool) -> bool>(&self, f: T) -> Self {
        match &self.val {
            Val::Boolean(b) => Self::from(f(*b)),

            _ => self.into_bool().mutate_bool(f),
        }
    }
}

// Equivalent to sprintf function in the js version
impl Display for Dynamic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.val {
            Val::String(s) => write!(f, "{}", s),

            Val::Number(n) => write!(
                f,
                "{}",
                n.to_string_radix_round(10, Some(*OUTPUT_PRECISION), rug::float::Round::Nearest)
            ),

            Val::Boolean(b) => write!(f, "{}", if *b { 1 } else { 0 }),

            Val::Empty => write!(f, ""),
        }
    }
}

impl PartialEq for Dynamic {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match &self.val {
            Val::String(s) => match &other.val {
                Val::String(o) => s == o,
                Val::Number(n) => match Num::parse(s) {
                    Ok(s) => Num::with_val(*FLOAT_PRECISION, s) == n.clone(),
                    Err(_) => false,
                },
                Val::Boolean(b) => {
                    if *b {
                        s == "1"
                    } else {
                        s == "0"
                    }
                }
                _ => false,
            },

            Val::Number(n) => match &other.val {
                Val::String(s) => match Num::parse(s) {
                    Ok(s) => Num::with_val(*FLOAT_PRECISION, s) == n.clone(),
                    Err(_) => false,
                },
                Val::Number(o) => n == o,
                Val::Boolean(b) => {
                    if *b {
                        n.clone() == 1
                    } else {
                        n.is_zero()
                    }
                }
                _ => false,
            },

            Val::Boolean(b) => match &other.val {
                Val::Number(n) => {
                    if *b {
                        n.clone() == 1
                    } else {
                        n.is_zero()
                    }
                }
                Val::Boolean(n) => b == n,
                _ => false,
            },

            Val::Empty => other.val == Val::Empty,
        }
    }
}

impl<'a> From<&'a str> for Dynamic {
    fn from(v: &'a str) -> Self {
        Self {
            val: Val::String(v.to_owned()),
            cur: 1,
        }
    }
}

impl From<String> for Dynamic {
    fn from(v: String) -> Self {
        Self {
            val: Val::String(v),
            cur: 1,
        }
    }
}

impl From<Num> for Dynamic {
    fn from(v: Num) -> Self {
        Self {
            val: Val::Number(v),
            cur: 2,
        }
    }
}

impl From<bool> for Dynamic {
    fn from(v: bool) -> Self {
        Self {
            val: Val::Boolean(v),
            cur: 3,
        }
    }
}

impl Into<Node> for Dynamic {
    fn into(self) -> Node {
        match self.val {
            Val::String(st) => Node::String(st),
            Val::Number(nm) => Node::Number(nm),
            Val::Boolean(bl) => {
                Node::Number(Num::with_val(*FLOAT_PRECISION, if bl { 1 } else { 0 }))
            }
            _ => panic!("Cannot convert emtpy value into Node"),
        }
    }
}

pub struct Sequence<'a> {
    cstr: Vec<Dynamic>,
    length: Option<usize>,
    block: Node,
    _i: Option<isize>,
    env: Option<&'a mut Environment>,
    index: usize,
}

impl<'a> Sequence<'a> {
    pub fn from_iter<T, U>(iter: T, block: Node, length: Option<usize>) -> Self
    where
        T: Iterator<Item = U>,
        Dynamic: From<U>,
    {
        let v = iter.map(|n| Dynamic::from(n)).collect();
        Self {
            cstr: v,
            length,
            block,
            _i: None,
            env: None,
            index: 0,
        }
    }

    pub fn from_vec<T>(v: Vec<T>, block: Node, length: Option<usize>) -> Self
    where
        Dynamic: From<T>,
        T: Clone,
    {
        let v = v.iter().map(|n| Dynamic::from(n.clone())).collect();
        Self {
            cstr: v,
            length,
            block,
            _i: None,
            env: None,
            index: 0,
        }
    }

    pub fn set_env(&mut self, env: &'a mut Environment) {
        self.env = Some(env);
    }

    fn traverse_replace(&mut self, n: Node) -> Node {
        if self._i.is_none() {
            self._i = Some(self.index as isize - 2);
        }

        match &n {
            Node::Block(body, nm) => {
                let new_body = body
                    .iter()
                    .map(|n| self.traverse_replace(n.clone()))
                    .collect();
                Node::Block(new_body, nm.clone())
            }

            Node::String(_) => n,

            Node::Number(_) => n,

            Node::Variable(v) => {
                if v == "_" {
                    self._i = Some(self._i.unwrap() - 1);
                    self.cstr[(self._i.unwrap() + 1) as usize].clone().into()
                } else {
                    n
                }
            }

            Node::Group(body) => {
                let new_body = body
                    .iter()
                    .map(|n| self.traverse_replace(n.clone()))
                    .collect();
                Node::Group(new_body)
            }

            Node::Op(n, largs, rargs) => {
                let nl = largs
                    .iter()
                    .map(|n| self.traverse_replace(n.clone()))
                    .collect();
                let nr = rargs
                    .iter()
                    .map(|n| self.traverse_replace(n.clone()))
                    .collect();
                Node::Op(n.clone(), nl, nr)
            }
        }
    }

    #[inline(always)]
    fn _next(&mut self) -> Option<Dynamic> {
        if self.index < self.cstr.len() {
            self.index += 1;
            Some(self.cstr[self.index - 1].clone())
        } else {
            self.index += 1;
            let block = self.traverse_replace(self.block.clone());
            self._i = None;
            let mut e = (*self.env.as_ref().unwrap()).clone();
            let res = crate::parser::parse_node(&mut e, &block);
            self.cstr.push(res.clone());

            Some(res)
        }
    }
}

impl<'a> Iterator for Sequence<'a> {
    type Item = Dynamic;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length.is_none() {
            self._next()
        } else if self.length.unwrap() == self.cstr.len() {
            None
        } else {
            self._next()
        }
    }
}

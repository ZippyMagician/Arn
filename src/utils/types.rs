#![allow(dead_code)]

use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use super::env::Environment;
use super::num::{to_u32, Num};
use super::tokens::Node;
use crate::{FLOAT_PRECISION, OUTPUT_PRECISION};

// Shorthand for this monstrosity
pub type Env = Rc<RefCell<Environment>>;

// Inner value enum for dynamic type
#[derive(Clone, Debug)]
pub enum Val {
    String(String),

    Number(Num),

    Boolean(bool),

    Array(Box<Sequence>),

    Empty,
}

impl Hash for Val {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Val::String(s) => s.hash(state),
            Val::Number(n) => n.clone().floor().to_u32_saturating().unwrap().hash(state),
            Val::Boolean(b) => b.hash(state),
            Val::Array(s) => s.as_ref().clone().collect::<Vec<Dynamic>>().hash(state),
            Val::Empty => "".hash(state),
        }
    }
}

// Struct that represents types in Arn
#[derive(Clone, Debug, Hash)]
pub struct Dynamic {
    val: Val,
    cur: u8,
}

#[allow(clippy::clippy::wrong_self_convention)]
impl Dynamic {
    pub fn empty() -> Self {
        Self {
            val: Val::Empty,
            cur: 0,
        }
    }

    pub fn new(val: Val, cur: u8) -> Self {
        Self { val, cur }
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

            Val::Number(_) => Self {
                val: Val::String(format!("{}", self)),
                cur: 1,
            },

            Val::Boolean(b) => Self {
                val: Val::String(b.to_string()),
                cur: 1,
            },

            Val::Array(n) => Self {
                val: Val::String(
                    n.clone()
                        .next()
                        .unwrap_or_else(|| Dynamic::from(""))
                        .literal_string(),
                ),
                cur: 1,
            },

            Val::Empty => Self {
                val: Val::String(String::new()),
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

            Val::Array(n) => Self {
                val: Val::Number(Num::with_val(
                    *FLOAT_PRECISION,
                    Num::parse(
                        n.clone()
                            .next()
                            .unwrap_or_else(|| Dynamic::from(""))
                            .literal_string(),
                    )
                    .unwrap_or_else(|_| Num::parse("0").unwrap()),
                )),
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

            Val::Array(n) => Self {
                val: Val::Boolean(
                    n.clone()
                        .next()
                        .unwrap_or_else(|| Dynamic::from(false))
                        .literal_bool(),
                ),
                cur: 3,
            },

            Val::Empty => Self {
                val: Val::Boolean(false),
                cur: 3,
            },
        }
    }

    pub fn into_array(&self) -> Self {
        match &self.val {
            Val::String(s) => {
                if s.contains(' ') {
                    let iter = s.split(' ').map(Dynamic::from);
                    Self {
                        val: Val::Array(Box::new(Sequence::from_iter(
                            iter.clone(),
                            Node::String(String::new()),
                            Some(iter.count()),
                        ))),
                        cur: 4,
                    }
                } else {
                    let iter = s.chars().map(|n| Dynamic::from(n.to_string()));
                    Self {
                        val: Val::Array(Box::new(Sequence::from_iter(
                            iter.clone(),
                            Node::String(String::new()),
                            Some(iter.count()),
                        ))),
                        cur: 4,
                    }
                }
            }

            Val::Number(_) => Dynamic::from(format!("{}", self)).into_array(),

            Val::Boolean(_) => Dynamic::from(format!("{}", self)).into_array(),

            Val::Array(_) => self.clone(),

            Val::Empty => Self {
                val: Val::Array(Box::new(Sequence::from_vec::<String>(
                    &[],
                    Node::String(String::new()),
                    Some(0),
                ))),
                cur: 4,
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
    pub fn literal_array(self) -> Sequence {
        match &self.val {
            Val::Array(seq) => seq.as_ref().clone(),
            _ => self.into_array().literal_array(),
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

    #[inline]
    pub fn is_array(&self) -> bool {
        self.cur == 4
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

    // Mutate inner `Val::Array`
    pub fn mutate_array<T: FnOnce(&mut Sequence) -> &mut Sequence>(&mut self, f: T) -> Self {
        match &mut self.val {
            Val::Array(seq) => Self {
                val: Val::Array(Box::new(f(seq.as_mut()).clone())),
                cur: 4,
            },

            _ => self.into_array().mutate_array(f),
        }
    }

    // Convert to Node
    pub fn into_node(self) -> Node {
        match self.val {
            Val::String(s) => Node::String(s),
            Val::Number(n) => Node::Number(n),
            Val::Boolean(b) => Node::Number(Num::with_val(*FLOAT_PRECISION, if b { 1 } else { 0 })),
            Val::Array(s) => {
                if !s.is_finite() {
                    panic!("Cannot convert infinite sequence into Node");
                }

                let s = s.as_ref().clone();
                Node::Sequence(
                    s.cstr.iter().cloned().map(Dynamic::into_node).collect(),
                    Box::new(s.block),
                    s.length
                        .map(|n| Box::new(Node::Number(Num::with_val(*FLOAT_PRECISION, n)))),
                )
            }
            Val::Empty => unreachable!(),
        }
    }
}

// Equivalent to sprintf function in the js version
impl Display for Dynamic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.val {
            Val::String(s) => write!(f, "{}", s),

            Val::Number(n) => {
                let s = n.to_string_radix_round(
                    10,
                    Some(*OUTPUT_PRECISION),
                    rug::float::Round::Nearest,
                );

                write!(
                    f,
                    "{}",
                    if s.contains('.') && !s.contains('e') {
                        // First remove trailing zeros, then the dot if that was everything
                        s.trim_end_matches('0')
                            .trim_end_matches('.')
                            .replace('-', "_")
                    } else {
                        s.replace('-', "_")
                    }
                )
            }

            Val::Boolean(b) => write!(f, "{}", if *b { 1 } else { 0 }),

            Val::Array(seq) => {
                let seq = seq.as_ref().clone();
                let is_infinite = !seq.is_finite();

                for entry in seq {
                    let mut as_string = format!("{}", entry);

                    if entry.is_array() {
                        as_string = as_string.replace('\n', " ");
                    }

                    if is_infinite {
                        println!("{}", as_string);
                    } else {
                        writeln!(f, "{}", as_string)?;
                    }
                }

                Ok(())
            }

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

            Val::Array(_) => todo!(),

            Val::Empty => matches!(other.val, Val::Empty),
        }
    }
}

// Technically this isn't true
impl Eq for Dynamic {}

impl PartialOrd for Dynamic {
    #[inline]
    fn partial_cmp(&self, other: &Dynamic) -> Option<Ordering> {
        match &self.val {
            Val::String(s) => match &other.val {
                Val::String(o) => s.partial_cmp(o),
                Val::Number(n) => match Num::parse(s) {
                    Ok(s) => Num::with_val(*FLOAT_PRECISION, s).partial_cmp(n),
                    Err(_) => None,
                },
                Val::Boolean(b) => {
                    if *b {
                        s.partial_cmp(&"1".to_string())
                    } else {
                        s.partial_cmp(&"0".to_string())
                    }
                }
                _ => s.partial_cmp(&other.clone().literal_string()),
            },

            Val::Number(n) => match &other.val {
                Val::String(s) => match Num::parse(s) {
                    Ok(s) => n.partial_cmp(&Num::with_val(*FLOAT_PRECISION, s)),
                    Err(_) => None,
                },
                Val::Number(o) => n.partial_cmp(o),
                Val::Boolean(b) => {
                    if *b {
                        n.partial_cmp(&1)
                    } else {
                        n.cmp0()
                    }
                }
                _ => n.partial_cmp(&other.clone().literal_num()),
            },

            Val::Boolean(b) => match &other.val {
                Val::Number(n) => {
                    if *b {
                        Num::with_val(*FLOAT_PRECISION, 1).partial_cmp(&1)
                    } else {
                        Num::new(*FLOAT_PRECISION).partial_cmp(n)
                    }
                }
                Val::Boolean(n) => b.partial_cmp(n),
                _ => b.partial_cmp(&other.clone().literal_bool()),
            },

            Val::Array(s) => match &other.val {
                Val::Array(o) => {
                    let s = s.as_ref().clone().count();
                    let o = o.as_ref().clone().count();

                    s.partial_cmp(&o)
                }

                _ => s
                    .as_ref()
                    .clone()
                    .collect::<Vec<_>>()
                    .partial_cmp(&other.clone().literal_array().collect::<Vec<_>>()),
            },

            Val::Empty => None,
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

impl<'a, T> From<&'a [T]> for Dynamic
where
    Dynamic: From<T>,
    T: Clone,
{
    fn from(v: &'a [T]) -> Self {
        Self {
            val: Val::Array(Box::new(Sequence::from_vec(
                v,
                Node::Block(vec![], None),
                Some(v.len()),
            ))),
            cur: 4,
        }
    }
}

impl<T> From<Vec<T>> for Dynamic
where
    Dynamic: From<T>,
    T: Clone,
{
    fn from(v: Vec<T>) -> Self {
        Self {
            val: Val::Array(Box::new(Sequence::from_vec(
                &v,
                Node::Block(vec![], None),
                Some(v.len()),
            ))),
            cur: 4,
        }
    }
}

impl<T, const N: usize> From<[T; N]> for Dynamic
where
    Dynamic: From<T>,
    T: Clone,
{
    fn from(v: [T; N]) -> Self {
        Self {
            val: Val::Array(Box::new(Sequence::from_vec(
                &v,
                Node::Block(vec![], None),
                Some(N),
            ))),
            cur: 4,
        }
    }
}

impl From<Sequence> for Dynamic {
    fn from(v: Sequence) -> Self {
        Self {
            val: Val::Array(Box::new(v)),
            cur: 4,
        }
    }
}

#[allow(clippy::from_over_into)]
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

#[derive(Clone, Debug)]
pub struct Sequence {
    pub cstr: Vec<Dynamic>,
    pub length: Option<usize>,
    pub block: Node,
    unparsed_length: Option<Node>,
    t_i: Option<isize>,
    env: Option<Env>,
    index: usize,
}

impl Sequence {
    pub fn from_iter<T, U>(iter: T, block: Node, length: Option<usize>) -> Self
    where
        T: Iterator<Item = U>,
        Dynamic: From<U>,
    {
        let v = iter.map(Dynamic::from).collect();
        Self {
            cstr: v,
            unparsed_length: None,
            length,
            block,
            t_i: None,
            env: None,
            index: 0,
        }
    }

    pub fn from_vec<T>(v: &[T], block: Node, length: Option<usize>) -> Self
    where
        Dynamic: From<T>,
        T: Clone,
    {
        let v = v.iter().map(|n| Dynamic::from(n.clone())).collect();
        Self {
            cstr: v,
            unparsed_length: None,
            length,
            block,
            t_i: None,
            env: None,
            index: 0,
        }
    }

    pub fn from_vec_dyn(v: &[Dynamic], block: Node, length: Option<usize>) -> Self {
        Self {
            cstr: v.to_owned(),
            unparsed_length: None,
            length,
            block,
            t_i: None,
            env: None,
            index: 0,
        }
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.length.is_some() || self.unparsed_length.is_some()
    }

    #[inline]
    pub fn len(&self) -> Option<usize> {
        self.length
    }

    #[inline]
    pub fn set_env(&mut self, env: Env) {
        self.env = Some(Rc::new(env.as_ref().clone()));
    }

    #[inline]
    pub fn set_env_self(self, env: Env) -> Self {
        Self {
            cstr: self.cstr,
            unparsed_length: self.unparsed_length,
            length: self.length,
            block: self.block,
            t_i: self.t_i,
            env: Some(env),
            index: self.index,
        }
    }

    fn traverse_replace(&mut self, n: Node) -> Node {
        let mut vals = self.cstr.iter().cloned().map(|n| n.into_node()).collect();
        super::traverse_replace(&mut vals, n)
    }

    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    fn _next(&mut self) -> Option<Dynamic> {
        if self.index < self.cstr.len() {
            self.index += 1;
            Some(self.cstr[self.index - 1].clone())
        } else {
            self.index += 1;
            let block = self.traverse_replace(self.block.clone());
            self.env
                .as_ref()
                .unwrap()
                .borrow_mut()
                .define_var("p", Dynamic::from(self.cstr.as_slice()));
            let res = crate::parser::parse_node(Rc::clone(self.env.as_ref().unwrap()), &block);
            self.cstr.push(res.clone());

            Some(res)
        }
    }
}

impl Iterator for Sequence {
    type Item = Dynamic;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.length.is_none() && self.unparsed_length.is_some() {
            self.length = Some(to_u32(
                self.env.as_ref().unwrap(),
                self.unparsed_length.as_ref().unwrap(),
            ) as usize);
        }

        if self.length.is_none() {
            self._next()
        } else if self.index == self.length.unwrap() {
            None
        } else {
            self._next()
        }
    }
}

impl DoubleEndedIterator for Sequence {
    fn next_back(&mut self) -> Option<Dynamic> {
        if self.len().is_none() {
            panic!("Can only implement DoubleEndedIterator for a finite Sequence");
        }

        while let Some(_) = self.next() {
            // Build the values, the starting index is initialized
            self.t_i = Some(self.index as isize - 1);
        }

        if self.t_i.unwrap() < 0 {
            None
        } else {
            self.t_i = Some(self.t_i.unwrap() - 1);
            Some(self.cstr[(self.t_i.unwrap() + 1) as usize].clone())
        }
    }
}

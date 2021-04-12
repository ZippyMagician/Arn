#![allow(dead_code)]

use std::fmt::{self, Display, Formatter};

use super::num::{Num, FLOAT_PRECISION, PRINT_PRECISION};

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
                    FLOAT_PRECISION,
                    Num::parse(s).unwrap_or_else(|_| Num::parse("0").unwrap()),
                )),
                cur: 2,
            },

            Val::Number(_) => self.clone(),

            Val::Boolean(b) => Self {
                val: Val::Number(Num::with_val(FLOAT_PRECISION, if *b { 1 } else { 0 })),
                cur: 2,
            },

            Val::Empty => Self {
                val: Val::Number(Num::with_val(FLOAT_PRECISION, 0)),
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
                n.to_string_radix_round(10, Some(PRINT_PRECISION), rug::float::Round::Nearest)
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
                    Ok(s) => Num::with_val(FLOAT_PRECISION, s) == n.clone(),
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
                    Ok(s) => Num::with_val(FLOAT_PRECISION, s) == n.clone(),
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

#[cfg(test)]
mod typing_tests {
    use super::*;

    #[test]
    fn test_casting() {
        let mut ty = Dynamic::empty().into_string();

        ty.mutate_string(|s| s + "57");
        assert_eq!(ty.val, Val::String(String::from("57")));

        ty.as_num();
        if let Val::Number(n) = &ty.val {
            assert_eq!(n.clone(), 57);
        } else {
            // Fail
            assert!(false)
        }

        assert_eq!(format!("{}", ty.into_bool()), "1");
    }
}

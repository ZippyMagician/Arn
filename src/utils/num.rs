use rug::Float;

use crate::FLOAT_PRECISION;

// Alias
pub type Num = Float;

#[derive(Debug, Clone, Copy)]
pub struct EmptyError();

impl<T> From<T> for EmptyError
where
    T: std::error::Error,
{
    #[inline]
    fn from(_: T) -> Self {
        Self()
    }
}

pub fn is_arn_num(string: &str) -> bool {
    if string.is_empty() {
        return false;
    }

    let mut seen_e = false;

    for (i, chr) in string.chars().enumerate() {
        match chr {
            '_' => {
                if i > 0 && string.chars().nth(i - 1).unwrap() != 'e' {
                    return false;
                }
            }

            'e' => {
                if seen_e {
                    return false;
                }

                seen_e = true;
            }

            '0'..='9' => {}

            _ => return false,
        }
    }

    if seen_e && string.matches('_').count() > 2 {
        return false;
    }

    string.matches('_').count() <= 1
}

pub fn parse_arn_num(string: &str) -> Result<Num, EmptyError> {
    let mut num = String::with_capacity(string.len());

    for (i, chr) in string.chars().enumerate() {
        match chr {
            '_' => {
                if i > 0 && string.chars().nth(i - 1).unwrap() != 'e' {
                    return Err(EmptyError());
                }
                num.push('-');
            }

            'e' => {
                if num.is_empty() || num.starts_with('-') {
                    num.push('1');
                }
                num.push('e');
            }

            '0'..='9' => num.push(chr),

            _ => return Err(EmptyError()),
        }
    }

    Ok({
        let num = Num::parse(&num)?;
        Num::with_val(*FLOAT_PRECISION, num)
    })
}

#[inline]
pub fn to_u32(env: &super::types::Env, n: &super::tokens::Node) -> u32 {
    crate::parser::parse_node(std::rc::Rc::clone(env), n)
        .literal_num()
        .floor()
        .to_u32_saturating_round(rug::float::Round::Down)
        .unwrap()
}

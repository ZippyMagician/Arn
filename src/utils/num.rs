use rug::Float;

use crate::FLOAT_PRECISION;

// Alias
pub type Num = Float;

pub fn is_arn_num(string: &str) -> bool {
    let count_ = string.matches('_').count();
    if string.is_empty() || count_ > 2 {
        return false;
    }

    let mut expos = None;

    for (i, chr) in string.char_indices() {
        match chr {
            '_' => {
                if i > 0 && Some(i - 1) != expos {
                    return false;
                }
            }

            'e' => {
                if expos.is_some() {
                    return false;
                }

                expos = Some(i);
            }

            '0'..='9' => {}

            _ => return false,
        }
    }

    if expos.is_some() {
        true
    } else {
        count_ <= 1
    }
}

pub fn parse_arn_num(string: &str) -> Result<Num, Box<dyn std::error::Error>> {
    let mut num = String::with_capacity(string.len() + 1);
    if string.starts_with('e') {
        num.push('1');
        num.push_str(string);
    } else if string.starts_with("_e") {
        num.push_str("-1");
        num.push_str(&string[1..]);
    } else {
        num.push_str(string);
    }
    num = num.replace('_', "-");

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

use super::{consts::CODEPAGE, num::Num};

pub fn pack(code: &str) -> String {
    let code = code.replace('\n', "\\n").chars().collect::<Vec<_>>();
    let bytes = code.iter().map(|r| (*r as u8 as i32 - 32) as u8);

    let bytes = pack_bytes(bytes);
    return bytes
        .iter()
        .map(|r| CODEPAGE[*r as usize].to_string())
        .collect::<Vec<_>>()
        .join("");
}

#[inline]
fn pack_bytes<T>(bytes: T) -> Vec<u8>
where
    T: Iterator<Item = u8> + DoubleEndedIterator,
{
    let mut result = Vec::new();
    let mut big = Num::new(1000);

    for byte in bytes.rev() {
        big = big * 95 + byte;
    }

    while !big.is_zero() {
        result.push((big.clone() % 256_u16).floor().to_u32_saturating().unwrap() as u8);
        big = (big / 256_u16).floor();
    }

    result
}

pub fn unpack(packed: &str) -> String {
    let code = packed.chars().collect::<Vec<_>>();
    let bytes = code
        .iter()
        .map(|r| CODEPAGE.iter().position(|n| *n == *r).unwrap_or(0) as u8);

    let bytes = unpack_bytes(bytes);
    bytes
        .iter()
        .map(|r| String::from_utf8(vec![*r + 32]).unwrap())
        .collect::<Vec<_>>()
        .join("")
        .replace("\\n", "\n")
}

#[inline]
fn unpack_bytes<T>(bytes: T) -> Vec<u8>
where
    T: Iterator<Item = u8> + DoubleEndedIterator,
{
    let mut result = Vec::new();
    let mut big = Num::new(1000);

    for byte in bytes.rev() {
        big = big * 256 + byte;
    }

    while !big.is_zero() {
        result.push((big.clone() % 95_u8).floor().to_u32_saturating().unwrap() as u8);
        big = (big / 95_u8).floor();
    }

    result
}

#[inline]
pub fn is_packed(code: &str) -> bool {
    unpack(&pack(code)) != code
}

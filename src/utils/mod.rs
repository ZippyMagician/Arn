pub mod compress;
pub mod consts;
pub mod env;
pub mod num;
pub mod tokens;
pub mod types;

pub fn create_str_range(s: usize, n: usize) -> String {
    (s..=n).map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
}

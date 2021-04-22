// This file was ported directly from js, so excuse the mess

use super::consts::COMPRESSED_CHARS;

fn capitalize(word: &str, existing: bool) -> String {
    if existing {
        word.to_owned()
    } else {
        word.chars().next().unwrap().to_uppercase().to_string() + &word[1..]
    }
}

pub fn decompress(chars: &str, all_cap: bool) -> String {
    let dictionary = include_str!("../../dictionary.txt")
        .split("\n")
        .filter(|n| !n.is_empty())
        .collect::<Vec<_>>();
    let s = chars.trim().chars().collect::<Vec<char>>();

    let mut decomp = String::new();
    let mut chr = 0;

    while chr < s.len() {
        let first = s[chr];
        let second = *s.get(chr + 1).unwrap_or(&'\u{0000}'); // lol
        if COMPRESSED_CHARS.contains(&first) {
            if COMPRESSED_CHARS.contains(&second) {
                decomp.push_str(&capitalize(
                    dictionary[COMPRESSED_CHARS.iter().position(|c| *c == first).unwrap() * 100
                        + COMPRESSED_CHARS.iter().position(|c| *c == second).unwrap()]
                    .trim(),
                    !all_cap && !decomp.is_empty(),
                ));
            } else {
                decomp.push_str(&format!(
                    "{}{}",
                    capitalize(
                        dictionary
                            [COMPRESSED_CHARS.iter().position(|c| *c == first).unwrap() * 100]
                            .trim(),
                        !all_cap && !decomp.is_empty()
                    ),
                    second
                ));
            }
            chr += 2;
        } else {
            chr += 1;
            decomp.push_str(&first.to_string());
        }
    }

    decomp
}

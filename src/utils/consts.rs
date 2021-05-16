// An easier way to create the precedence and operator global constants
macro_rules! operators {
    ($($chr:literal : $prec:literal; $left_rank:literal - $right_rank:literal),*) => {
        #[derive(Clone)]
        pub struct Operators {
            pub precedence: std::collections::HashMap<String, i32>,
            pub operators: [String; len!($($chr),*)],
            pub rank: std::collections::HashMap<String, (i32, i32)>,
        }

        impl Operators {
            pub fn new() -> Self {
                Self {
                    precedence: hashmap![i32,
                        $(
                            $chr => $prec
                        ),*
                    ],
                    operators: [
                        $(
                            $chr.to_string()
                        ),*
                    ],
                    rank: hashmap![(i32, i32),
                        $(
                            $chr => ($left_rank, $right_rank)
                        ),*
                    ]
                }
            }
        }
    };
}

// Get length based on number of items inputted
macro_rules! len {
    () => { 0 };
    ($item:literal) => { 1 };
    ($item:literal, $($extras:literal),*) => { 1 + len!($($extras),*) }
}

macro_rules! hashmap {
    ($type:ty, $($key:literal => $val:literal),*) => {{
        use std::collections::HashMap;
        let mut hash: HashMap<String, $type> = HashMap::new();

        $(
            hash.insert($key.to_string(), $val);
        )*

        hash
    }};

    ($type:ty, $($key:literal => $val:expr),*) => {{
        use std::collections::HashMap;
        let mut hash: HashMap<String, $type> = HashMap::new();

        $(
            hash.insert($key.to_string(), $val);
        )*

        hash
    }};
}

// REWORKS/PATCHES NEEDED: `;`, `\`, `@`
// REMOVED: `n_`
// UNUSED: `!!`
// NEEDS CHANGING: numbers

// Little macro I created to make the global Operators class much nicer.
// First number is precedence, second is left # of args, third is right # of args
operators! {
    '.': 11; 1-1,
    '^': 10; 1-1, "<>": 10; 1-1,
    '*': 9; 1-1, '/': 9; 1-1,
    '%': 8; 1-1,
    ":|": 7; 1-1, ":!": 7; 1-1,
    '+': 6; 1-1, '-': 6; 1-1, ".$": 6; 1-1,
    ".~": 5; 1-0, "=>": 5; 1-1, "->": 5; 1-1, '~': 5; 0-1, '#': 5; 1-0, ';': 5; 1-1, ":_": 5; 1-0, ":%": 5; 1-0, ".|": 5; 1-0, ".<": 5; 1-0, "..": 5; 1-0, ".=": 5; 1-0,
    ":n": 4; 1-0, ":s": 4; 1-0, ":}": 4; 1-0, ":{": 4; 1-0, ".}": 4; 1-0, ".{": 4; 1-0, ":@": 4; 1-0, "^*": 4; 1-0, "&.": 4; 0-3, ":i": 4; 1-1,
    '!': 4; 0-1, ":v": 4; 0-1, ":^": 4; 0-1, "++": 4; 0-1, "--": 4; 0-1, ":*": 4; 0-1, ":/": 4; 0-1,
    ":+": 4; 0-1, ":-": 4; 0-1, ":>": 4; 0-1, ":<": 4; 0-1, "|:": 4; 0-1, "?.": 4; 0-1, "#.": 4; 0-1, "*.": 4; 0-1,
    "$.": 4; 0-1, 'z': 4; 1-1, "#>": 4; 0-1, "#:": 4; 0-1, '?': 4; 1-1, "!.": 4; 0-1,
    '|': 3; 1-1,
    '=': 2; 1-1, "!=": 2; 1-1, '<': 2; 1-1, "<=": 2; 1-1, '>': 2; 1-1, ">=": 2; 1-1,
    "&&": 1; 1-1, "||": 1; 1-1,
    ':': 0; 1-1, "::": 0; 1-1, "??": 0; 1-2, '@': 0; 1-1, '&': 0; 1-1, '$': 0; 0-2, "$:": 0; 0-2, "/:": 0; 0-2, "\\": 0; 1-1, ":\\": 0; 1-1,
    ":=": -1; 1-1
}

lazy_static! {
    pub static ref OPTIONS: Operators = Operators::new();
    pub static ref CODEPAGE: Vec<char> = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~¡¢£¤¥¦§¨©ª«¬®¯°○■↑↓→←║═╔╗╚╝░▒►◄│─┌┐└┘├┤┴┬♦┼█▄▀▬±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿŒœŠšŸŽžƒƥʠˆ˜–—‘’‚“”„†‡•…‰‹›€™⁺⁻⁼⇒⇐★Δ".chars().collect();
    pub static ref COMPRESSED_CHARS: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ`1234567890-=[]\\;'/~@#$%^&*()_+{}|\"<>".chars().collect();
}

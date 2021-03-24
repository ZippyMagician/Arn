// An easier way to create the precedence and operator global constants
#[macro_export]
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
    ($item:literal, $($extras:literal),*) => { 1 + len!($($extras),*); }
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

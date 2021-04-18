# Anvil
Current status of the Rust interpreter for Arn
- [x] Place implied `_` into program -> first pass
- [x] Convert input stream into postfix notation for very easy parsing -> second pass
- [x] Change how numbers are parsed to match with Javascript version
- [x] Convert the postfix program stream into an AST -> third pass
- [x] Struct that represents Arn's dynamic typing
- [x] Environment for storing variables and functions
- [x] Parser that takes in AST
- [x] Fix found issues
 
 MILESTONE: move to `Arn` repository in new branch
- [x] Implement Sequences / Arrays
- [x] Implement Sequence-related fixes
- [x] Figure out how to implement `;`, `\`, and `@`
- [x] Implement command line arguments
- [ ] Implement compressed strings and Carn
- [ ] Fix found issues

Current (found) issues
- [x] Doesn't place `_` inside blocks
- [x] Can only parse single expressions
  * Expressions separated by `,`

## Building
See [here](https://docs.rs/gmp-mpfr-sys/1.4.4/gmp_mpfr_sys/index.html#building-on-gnulinux) for requirements. Once installed, run
```
cargo install --path path/to/repository
```

## Changes
There were some changes made to the language to reduce bloat, as it is a golfing language at heart
  - **No More `:` or `:=`**: You can't assign variables or functions any more
  - **No More 2+ Arg Functions**: All functions are called with `<ARG> . <FN>`

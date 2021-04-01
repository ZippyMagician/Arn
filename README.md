# Anvil
Current status of the Rust interpreter for Arn
- [x] Place implied `_` into program -> first pass
- [x] Convert input stream into postfix notation for very easy parsing -> second pass
- [x] Change how numbers are parsed to match with Javascript version
- [ ] Convert the postfix program stream into an AST -> third pass
- [ ] Environment for storing variables
- [ ] Parser that takes in AST -> type variability will be tricky
- [ ] Fix found issues
- [ ] Fix bugs
 
 MILESTONE: move to `Arn` repository in new branch
- [ ] Implement Sequences / Arrays
- [ ] Figure out how to implement `;`, `\`, and `@`
- [ ] Fix found issues
- [ ] Fix bugs

Current (found) issues
- [x] Doesn't place `_` inside blocks
- [ ] Can only parse single expressions

## Building
See [here](https://docs.rs/gmp-mpfr-sys/1.4.4/gmp_mpfr_sys/index.html#building-on-gnulinux) for requirements. Once installed, run
```
cargo install --path path/to/repository
```
# Arn
[![Build Status](https://travis-ci.org/ZippyMagician/Arn.svg?branch=master)](https://travis-ci.org/ZippyMagician/Arn)

A general-purpose functional golfing language. [Tutorial](https://github.com/ZippyMagician/Arn/wiki/Tutorial)

## Installation
### Post 1.0
First, ensure [Rust](https://rust-lang.org) is installed on your system and that [these requirements](https://docs.rs/gmp-mpfr-sys/1.4.4/gmp_mpfr_sys/index.html#building-on-gnulinux) are fulfilled.
You can either build from source by cloning the repository and then running
```
cargo install --path path/to/repository
```
for the latest features, or by simply running
```
cargo install arn-language
```
for the current release edition. You then can run
```
arn --help
```
to get a list of commands.
### Prior to 1.0
To install **Arn** you must have [Node.js](https://nodejs.org) installed on your system. Once installed, run
```sh
npm install -g arn-language
```
You can then use
```sh
arn help
```
to get a list of commands
## About
**Arn** is a golfing language; that is, it is designed to perform tasks in as few bytes as possible. It draws heavy inspiration from **J**/**APL**

Arn is constructed of variable declarations, functions, and symbols. These symbols come in the forms of prefixes, infixes, and suffixes. A full syntax and description can be found at [this page](https://github.com/ZippyMagician/Arn/wiki).
This format, however, may lead to instances where your program needs to be a few bytes shorter in order to compete. This is where **Carn** (Compressed Arn) comes in.

### Compression
**Carn** is the compressed version of **Arn**. The interpeter has the ability to distinguish between these two program formats and interpret each separately, without any input from the user. Carn is encoded using its own Code Page, based on __CP1252__. It can be found below. The Arn interpreter will compress your program by passing in the `-c` flag to the compiler through the command line.

#### Code Page
| `_` | `_0` | `_1` | `_2` | `_3` | `_4` | `_5` | `_6` | `_7` | `_8` | `_9` | `_A` | `_B` | `_C` | `_D` | `_E` | `_F` |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---:
| **`0_`** | `!` | `"` | `#` | `$` | `%` | `&` | `'` | `(` | `)` | `*` | `+` | `,` | `-` | `.` | `/` | `0` |
| **`1_`** | `1` | `2` | `3` | `4` | `5` | `6` | `7` | `8` | `9` | `:` | `;` | `<` | `=` | `>` | `?` | `@` |
| **`2_`** | `A` | `B` | `C` | `D` | `E` | `F` | `G` | `H` | `I` | `J` | `K` | `L` | `M` | `N` | `O` | `P` |
| **`3_`** | `Q` | `R` | `S` | `T` | `U` | `V` | `W` | `X` | `Y` | `Z` | `[` | `\` | `]` | `^` | `_` | ``` ` ``` |
| **`4_`** | `a` | `b` | `c` | `d` | `e` | `f` | `g` | `h` | `i` | `j` | `k` | `l` | `m` | `n` | `o` | `p` |
| **`5_`** | `q` | `r` | `s` | `t` | `u` | `v` | `w` | `x` | `y` | `z` | `{` | `\|` | `}` | `~` | `¡` | `¢` |
| **`6_`** | `£` | `¤` | `¥` | `¦` | `§` | `¨` | `©` | `ª` | `«` | `¬` | `®` | `¯` | `°` | `○` | `■` | `↑` |
| **`7_`** | `↓` | `→` | `←` | `║` | `═` | `╔` | `╗` | `╚` | `╝` | `░` | `▒` | `►` | `◄` | `│` | `─` | `┌` |
| **`8_`** | `┐` | `└` | `┘` | `├` | `┤` | `┴` | `┬` | `♦` | `┼` | `█` | `▄` | `▀` | `▬` | `±` | `²` | `³` |
| **`9_`** | `´` | `µ` | `¶` | `·` | `¸` | `¹` | `º` | `»` | `¼` | `½` | `¾` | `¿` | `À` | `Á` | `Â` | `Ã` |
| **`A_`** | `Ä` | `Å` | `Æ` | `Ç` | `È` | `É` | `Ê` | `Ë` | `Ì` | `Í` | `Î` | `Ï` | `Ð` | `Ñ` | `Ò` | `Ó` |
| **`B_`** | `Ô` | `Õ` | `Ö` | `×` | `Ø` | `Ù` | `Ú` | `Û` | `Ü` | `Ý` | `Þ` | `ß` | `à` | `á` | `â` | `ã` |
| **`C_`** | `ä` | `å` | `æ` | `ç` | `è` | `é` | `ê` | `ë` | `ì` | `í` | `î` | `ï` | `ð` | `ñ` | `ò` | `ó` |
| **`D_`** | `ô` | `õ` | `ö` | `÷` | `ø` | `ù` | `ú` | `û` | `ü` | `ý` | `þ` | `ÿ` | `Œ` | `œ` | `Š` | `š` |
| **`E_`** | `Ÿ` | `Ž` | `ž` | `ƒ` | `ƥ` | `ʠ` | `ˆ` | `˜` | `–` | `—` | `‘` | `’` | `‚` | `“` | `”` | `„` |
| **`F_`** | `†` | `‡` | `•` | `…` | `‰` | `‹` | `›` | `€` | `™` | `⁺` | `⁻` | `⁼` | `⇒` | `⇐` | `★` | `Δ` |

### Some sample programs
#### Hello, World
```
'yt, bs!
```
#### Cat program
```
_
```
#### FizzBuzz (1->100)
```
~e2@"Fizz"^!%3|`#&`^!%5||
```
#### Fibonacci Sequence
```
[1 1{+
```
#### Prime Check
```
#.:}=
```
*Uses Wilson's Theorem*
```
!(f/+1)%
```
### Future Plans
I plan on working on more practical features in the future, and I'm also going to look into changing the way certain operations work on sequences, among other things.

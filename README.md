I am currently in the process of completely re-writing the Arn parser in order to fix all the issues the current one has (op precedence being the primary offender). See [here](https://github.com/ZippyMagician/Anvil). When it gets close to release I will move it into a new branch of this repository (new-parser). Please be aware this project is not dead, I've just been avoiding it due to the current state of the code.

# Arn
[![Build Status](https://travis-ci.org/ZippyMagician/Arn.svg?branch=master)](https://travis-ci.org/ZippyMagician/Arn)

A general-purpose function golfing language.

## Installation
To install **Arn** you must have [Node.js](https://nodejs.org) installed on your system. Once installed, run
```sh
npm install -g arn-language
```
You can then run the command
```sh
arn run PATH
```
to run a file as an Arn program. You can also pass extra arguments to pass some user input to the program (example below). Use `arn help` to get a full list of commands/flags.
```sh
arn run PATH 5 "Hello, World!"
```
Would pass two lines of input to the program, one with a 5, and one with the string `Hello, World!`

## About
**Arn** is a golfing language; that is, it is designed to perform tasks in as few bytes as possible. However, unlike other golfing languages (such as **05AB1E** or **Gaia**), Arn is a functional paradigm with variable-based storage.
This is different from other golfing languages, which mainly use single-character commands. Arn is much more similar to **J**, therefore, than any of these other golfing languages.

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
| **`3_`** | `Q` | `R` | `S` | `T` | `U` | `V` | `W` | `X` | `Y` | `Z` | `[` | `\` | `]` | `^` | `_` | ``` |
| **`4_`** | `a` | `b` | `c` | `d` | `e` | `f` | `g` | `h` | `i` | `j` | `k` | `l` | `m` | `n` | `o` | `p` |
| **`5_`** | `q` | `r` | `s` | `t` | `u` | `v` | `w` | `x` | `y` | `z` | `{` | `|` | `}` | `~` | `¡` | `¢` |
| **`6_`** | `£` | `¤` | `¥` | `¦` | `§` | `¨` | `©` | `ª` | `«` | `¬` | `®` | `¯` | `°` | `○` | `■` | `↑` |
| **`7_`** | `↓` | `→` | `←` | `║` | `═` | `╔` | `╗` | `╚` | `╝` | `░` | `▒` | `►` | `◄` | `│` | `─` | `┌` |
| **`8_`** | `┐` | `└` | `┘` | `├` | `┤` | `┴` | `┬` | `♦` | `┼` | `█` | `▄` | `▀` | `▬` | `±` | `²` | `³` |
| **`9_`** | `´` | `µ` | `¶` | `·` | `¸` | `¹` | `º` | `»` | `¼` | `½` | `¾` | `¿` | `À` | `Á` | `Â` | `Ã` |
| **`A_`** | `Ä` | `Å` | `Æ` | `Ç` | `È` | `É` | `Ê` | `Ë` | `Ì` | `Í` | `Î` | `Ï` | `Ð` | `Ñ` | `Ò` | `Ó` |
| **`B_`** | `Ô` | `Õ` | `Ö` | `×` | `Ø` | `Ù` | `Ú` | `Û` | `Ü` | `Ý` | `Þ` | `ß` | `à` | `á` | `â` | `ã` |
| **`C_`** | `ä` | `å` | `æ` | `ç` | `è` | `é` | `ê` | `ë` | `ì` | `í` | `î` | `ï` | `ð` | `ñ` | `ò` | `ó` |
| **`D_`** | `ô` | `õ` | `ö` | `÷` | `ø` | `ù` | `ú` | `û` | `ü` | `ý` | `þ` | `ÿ` | `Œ` | `œ` | `Š` | `š` |
| **`E_`** | `Ÿ` | `Ž` | `ž` | `ƒ` | `ƥ` | `ʠ` | `ˆ` | `˜` | `–` | `—` | `‘` | `’` | `‚` | `“` | `”` | `„` |
| **`F_`** | `†` | `‡` | `•` | `…` | `‰` | `‹` | `›` | `€` | `™` | `⁺` | `⁻` | `⁼` | ` ` | ` ` | ` ` | ` ` |

### The future
Arn is still very early in development (as of writing this, version `0.1`!). This means that all features in Arn are subject to change. Any advice, feedback, or pull requests that improve the language are welcome.
Current features that will exist in the future:
 * Lots of command-line flags
    - These will have options on the online version
 * Rework text encoding

// All constant values used throughout the program for various purposes
// TODO: Make use of bignumber.js
const { default: BigNumber } = require('bignumber.js');

// All punctuation. It is stored so that different punctuation can go adjacent to one another, allowing for shorter programs.
module.exports.punctuation = [
    '!', '$', '#', '\\',                                                       // Single-length prefixes
    '=', '<', '>', '+', '-', '*', '/', '%', '^', '|', '@', '.', '~', '@', '&', // Single-length infixes
    '#', '?',                                                                  // Single-length suffixes
    '!!', ':v', ':^', '++', '--', ':*', ':/', ':>', ':<', '|:', '$:', 'n_',    // Double-length prefixes
    '<=', '>=', '!=', '||', '&&', ':|', '->', '=>', ':!', ':?', '::', '@:',    // Double-length infixes
    '*^', ':_', ':{', ':}', ':@',                                              // Double-length suffixes
    '{', '}', '(', ')', '[', ']', ',', ':=', ':', ':n', ':s', ':i', ';'        // Other punctuation
];

module.exports.prefixes = [
    'n_', '!', '$', '\\',
    '!!', ':v', ':^', '++', '--', ':*', ':/', ':>', ':<', '|:',  '$:'
];

module.exports.infixes = [
    '=', '<', '>', '+', '-', '*', '/', '%', '^', '|', '@', '.', '~', '@',
    '<=', '>=', '!=', '||', '&&', ':|', '->', '=>', ':!', ':?', '::', '@:',
    '?', ':=', ':', '&', ':i'
];

module.exports.suffixes = [
    '#',
    '*^', ':_', ':n', ':s', ':}', ':{', ':@'
];

// Infixes that cannot follow other infixes; they take priority
module.exports.ninfixes = [
    '=', '!=', '<', '>', '<=', '>=', '+', '-', '&'
]

// Builtin functions and their definition (or marked as true if hardcoded)
module.exports.builtins = {
    max: 1,
    min: 1,
    out: 1,
    outl: 1,
    in: 0,
    intr: 2,
    fact: 1,
    mean: 1,
    mode: 1
}

// Predefined variables
// Out of date
module.exports.vars = {
    'c': '""',
    pi: new BigNumber('3.14159265358979323846'),
    e: new BigNumber('2.71828182845904523536')
}
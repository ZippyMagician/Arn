// All constant values used throughout the program for various purposes
const { default: BigNumber } = require('bignumber.js');

// All punctuation. It is stored so that different punctuation can go adjacent to one another, allowing for shorter programs.
module.exports.punctuation = [
    'n', '!', '$', '#', '\\',                                             // Single-length prefixes
    '=', '<', '>', '+', '-', '*', '/', '%', '^', '|', '@', '.', '~', '@', '&', // Single-length infixes
    '#', '?',                                                                  // Single-length suffixes
    '!!', ':v', ':^', '++', '--', ':*', ':/', ':>', ':<', '|:', '$:',          // Double-length prefixes
    '<=', '>=', '!=', '||', '&&', ':|', '->', '=>', ':!', ':?', '::', '@:',    // Double-length infixes
    '*^', ':_',                                                                // Double-length suffixes
    '{', '}', '(', ')', '[', ']', ',', ':=', ':', ':n', ':s', ':i', ';'        // Other punctuation
];

module.exports.prefixes = [
    'n', '!', '$', '\\',
    '!!', ':v', ':^', '++', '--', ':*', ':/', ':>', ':<', '|:',  '$:'
];

module.exports.infixes = [
    '=', '<', '>', '+', '-', '*', '/', '%', '^', '|', '@', '.', '~', '@',
    '<=', '>=', '!=', '||', '&&', ':|', '->', '=>', ':!', ':?', '::', '@:',
    '?', ':=', ':', '&'
];

module.exports.suffixes = [
    '#',
    '*^', ':_', ':n', ':s', ':i'
];

// Infixes that cannot follow other infixes; they take priority
module.exports.ninfixes = [
    '=', '!=', '<', '>', '<=', '>=', '*', '/', '&'
]

// Builtin functions and their definition (or marked as true if hardcoded)
module.exports.builtins = {
    max: 1,
    min: 1,
    out: 1,
    outl: 1,
    in: 0,
    line: 0,
    map: 2,
    intr: 2,
    fact: 1,
    mean: 1,
    pop: 1,
    push: 2
}

// Predefined variables
module.exports.vars = {
    '\'': '""',
    pi: new BigNumber('3.14159265358979323846'),
    e: new BigNumber(' 2.71828182845904523536')
}
const arn = require('../src');
const { assert_eq, reset, Options } = require('./helper.js');
const { sprintf } = require('../src/formatter.js');

console.log("Beginning string tests...");

// Test 1
assert_eq(
    sprintf(arn.parse("'Mh└a└", new Options())),
    "Hello, World!"
);

// Test 2
assert_eq(
    sprintf(arn.parse(':{ | :}', new Options(["Greetings!"]))),
    "G!"
);

// Test 3
assert_eq(
    sprintf(arn.parse('"This is a string":s', new Options())),
    "This\nis\na\nstring"
);

reset();
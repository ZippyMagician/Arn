const arn = require('../src');
const { assert_eq, reset, Options } = require('./helper.js');
const { sprintf } = require('../src/formatter.js');

console.log("Beginning array tests...");

// Test 1
assert_eq(
    sprintf(arn.parse("${0=%2}1=>10", new Options())),
    "2\n4\n6\n8\n10"
);

// Test 2
assert_eq(
    sprintf(arn.parse("+\\[1 {+1} -> 10]", new Options())),
    "55"
);

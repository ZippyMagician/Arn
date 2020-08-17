const colors = require('colors');

module.exports.test_count = 0;

module.exports.assert_eq = (left, right) => {
    if (left === right) {
        console.log(`  * Test ${++module.exports.test_count}: ${"Passed".green}`);
    } else {
        throw new Error(`Test ${++module.exports.test_count} failed\n  Expected: ${right}, Got: ${left}`);
    }
}

module.exports.reset = () => {
    module.exports.test_count = 0;
}

module.exports.Options = class Options {
    constructor(long = [], stdin = " ", c = false, u = false, d = false) {
        this.long = long;
        this.stdin = stdin;
        this.c = c;
        this.u = u;
        this.d = d;
    }
}
const { pack, unpack, isPacked } = require('./compressor.js');
const { decompress } = require('./dictionary.js');
const { tokenize } = require('./lexer.js');
const { makeAST: to_ast } = require('./ast.js');
const { walkTree: parse } = require('./parse.js');

const { printf } = require('./formatter.js');

module.exports.run = (code, opts) => {
    if (opts.d) {
        console.log("Program:", code);
    }
    if (opts.c) {
        console.log("Packed:", pack(code));
        return;
    }
    if (isPacked(code.replace(/\n/g, " ").replace(/\s{2}/g, " "))) code = unpack(code);
    printf(parse(to_ast(tokenize(code)), opts));
}

module.exports.parse = (code, opts) => {
    if (isPacked(code)) code = unpack(code);
    return parse(to_ast(tokenize(code)), opts);
}
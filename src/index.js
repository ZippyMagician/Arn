const { pack, unpack, isPacked } = require('./compressor.js');
const { decompress } = require('./dictionary.js');
const { tokenize } = require('./lexer.js');
const { makeAST: toAST } = require('./ast.js');
const { walkTree: parse } = require('./parse.js');

const { printf } = require('./formatter.js');

module.exports.run = (code, opts) => {
    if (opts.d) {
        console.log("Program:", code);
        console.log("Tokens:", tokenize(code));
        console.log("AST:", JSON.stringify(toAST(tokenize(code), code)));
    }
    if (opts.c) {
        console.log("Packed:", pack(code));
        return;
    }

    if (isPacked(code.replace(/\n/g, " ").replace(/\s{2,}/g, " "))) code = unpack(code);
    if (opts.m) code = `{${code}}\\`;
    printf(parse(toAST(tokenize(code), code), opts, code));
}

module.exports.parse = (code, opts) => {
    if (isPacked(code)) code = unpack(code);
    if (opts.m) code = `{${code}}\\`;
    return parse(toAST(tokenize(code), code), opts, code);
}
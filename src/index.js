const compressor = require('./compressor.js');
const str_packer = require('./dictionary.js');
const tokenize = require('./lexer.js');
const to_ast = require('./ast.js');
const parse = require('./parse.js');

const { printf } = require('./formatter.js');

module.exports.run = (code, opts) => {
    if (opts.d) {
        console.log("Program:", code);
    }
    if (opts.c) {
        console.log("Packed:", compressor.pack(code));
        return;
    }
    if (compressor.isPacked(code)) code = compressor.unpack(code);
    
    printf(parse(to_ast(tokenize(code)), opts));
}

module.exports.parse = (code, opts) => {
    if (compressor.isPacked(code)) code = compressor.unpack(code);
    return parse(to_ast(tokenize(code)), opts);
}
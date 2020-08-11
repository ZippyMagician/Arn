const compressor = require('./compressor.js');
const str_packer = require('./dictionary.js');
const tokenize = require('./lexer.js');
const to_ast = require('./ast.js');
// TODO: Finish parser
const parse = require('./parse.js');

module.exports = (code, opts) => {
    if (opts.c) {
        console.log("Packed:", compressor.pack(code));
        return;
    }
    if (compressor.isPacked(code)) code = compressor.unpack(code);
    
    console.log(parse(to_ast(tokenize(code)), opts));
}
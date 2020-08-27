const constants = require('./constants.js');

// Version 1.1 as of 8/9/2020 4:43 PM EST
module.exports.tokenize = function tokenize(code) {
    let buffer = code.trim();
    let tokens = [];
    
    while (buffer.length) {
        let item;
        if (item = /^"((?:\\"|[^"])*)"?/g.exec(buffer)) {
            tokens.push({type: "string", value: item[1]});
            buffer = buffer.slice(item[0].length).trim();
        } else if (item = /^(['`])((?:\\['`]|[^'`])*)['`]?/g.exec(buffer)) {
            tokens.push({type: "string", char: item[1], value: item[2]});
            buffer = buffer.slice(item[0].length).trim();
        } else if (constants.punctuation.includes(buffer[0]) || constants.punctuation.includes(buffer.substr(0, 2))) {
            if (constants.punctuation.includes(buffer.substr(0, 2))) {
                tokens.push({type: "punctuation", value: buffer.substr(0, 2)});
                buffer = buffer.slice(2).trim();
            } else {
                tokens.push({type: "punctuation", value: buffer[0]});
                buffer = buffer.slice(1).trim();
            }
        } else if (item = /^((?:[0-9]*\.?[0-9]+)*e?[0-9]*\.?[0-9]+)/g.exec(buffer)) {
            tokens.push({type: "integer", value: item[1]});
            buffer = buffer.slice(item[0].length).trim();
        } else if (item = /^([a-zA-Z_][a-zA-Z0-9_]*)/g.exec(buffer)) {
            tokens.push({type: "variable", value: item[1]});
            buffer = buffer.slice(item[0].length).trim();
        } else {
            throw new Error("Did not recognize token in buffer: " + buffer);
        }
    }

    return tokens;
}
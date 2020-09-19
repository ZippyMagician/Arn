const constants = require('./constants.js');
const { constructArea } = require('./formatter.js');

function fix(str) {
    return str.replace(/\t/g, "    ");
}

// Version 1.1 as of 8/9/2020 4:43 PM EST
module.exports.tokenize = function tokenize(code) {
    let pos = 0;
    let line = 0;
    let buffer = fix(code.trim().replace(/\r\n/g, "\n"));
    // Need to do this multiple times in some cases
    while (/\n[ \t]+\n/g.test(buffer)) buffer = buffer.replace(/\n[ \t]+\n/g, "\n\n");
    let tokens = [];
    
    while (buffer.length) {
        let item;
        if (item = /^"((?:\\"|[^"])*)"?/g.exec(buffer)) {
            tokens.push({type: "string", value: item[1], pos, line});
            buffer = buffer.slice(item[0].length);
            pos += item[0].length + (buffer.length - buffer.trim().length);
        } else if (item = /^(['`])((?:\\['`]|[^'`])*)['`]?/g.exec(buffer)) {
            tokens.push({type: "string", char: item[1], value: item[2], pos, line});
            buffer = buffer.slice(item[0].length);
            pos += item[0].length + (buffer.length - buffer.trim().length);
        } else if (constants.punctuation.includes(buffer[0]) || constants.punctuation.includes(buffer.substr(0, 2))) {
            if (constants.punctuation.includes(buffer.substr(0, 2))) {
                tokens.push({type: "punctuation", value: buffer.substr(0, 2), pos, line});
                buffer = buffer.slice(2);
                pos += 2 + (buffer.length - buffer.trim().length);
            } else {
                tokens.push({type: "punctuation", value: buffer[0], pos, line});
                buffer = buffer.slice(1);
                pos += 1 + (buffer.length - buffer.trim().length);
            }
        } else if (item = /^((?:_?[0-9]*\.?[0-9]+)*e?_?[0-9]*\.?[0-9]+)/g.exec(buffer)) {
            tokens.push({type: "integer", value: item[1].replace(/_/g, "-"), pos, line});
            buffer = buffer.slice(item[0].length);
            pos += item[0].length + (buffer.length - buffer.trim().length);
        } else if (item = /^([a-zA-Z_][a-zA-Z0-9_]*)/g.exec(buffer)) {
            tokens.push({type: "variable", value: item[1], pos, line});
            buffer = buffer.slice(item[0].length);
            pos += item[0].length + (buffer.length - buffer.trim().length);
        } else {
            throw new Error("Did not recognize token in buffer.\n" + constructArea(code.trim(), line, tokens[tokens.length - 1].pos));
        }
        
        if ([...buffer].filter(r => r === "\n").length !== [...buffer.trim()].filter(r => r === "\n").length) {
            let store = [...buffer].filter(r => r === "\n").length - [...buffer.trim()].filter(r => r === "\n").length;
            line += store;
            pos = buffer.length - buffer.trim().length - store;
        }

        buffer = buffer.trim();
    }
    
    return tokens;
}
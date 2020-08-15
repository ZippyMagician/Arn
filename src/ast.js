const constants = require('./constants.js');

function getFoldLength(tokens, from) {
    let bIndex = from, fIndex = from;
    while (tokens[bIndex--] != {type: "punctuation", value: "{"} && bIndex > 0) {};
    bIndex += 1;
    if (bIndex > 0) {
        while (tokens[fIndex] !== {type: "punctuation", value: ";"} && fIndex > 0) fIndex--;
    }
    return [fIndex, bIndex - fIndex];
}

// Version 1 as of 8/6/2020 2:42 PM EST
module.exports = function makeAST(tokens) {
    const stream = tokens;
    let index = -1;
    const last = () => stream[index - 1];
    const look = () => stream[index];
    const peek = () => stream[index + 1];
    const next = () => stream[++index];

    let ast = {type: "prog", contents: []};

    function isPunc(char, val = false) {
        return (val || look()) && (val || look()).type === "punctuation" && (val || look()).value === char;
    }

    // TODO: Make sure this will work for all edge cases
    function validItem(obj) {
        return obj && obj.type !== "block" && obj.type !== "function"// && obj.type !== "infix";
    }

    function isFunction(key) {
        let mapped = ast.contents.filter(r => r.type === "function").map(r => [r.name, r.args.length]);
        for (let key in constants.builtins) mapped.push([key, constants.builtins[key]]);
        let res = mapped.filter(r => r[0] === key);

        return res.length !== 0 && res;
    }

    function parseContents(start = "{", end = "}") {
        let contents = [];
        let offset = 0;
        while (next() && (!isPunc(end) || offset)) {
            if (!offset && isPunc(end)) break;
            else if (isPunc(end)) offset--;
            else if (isPunc(start)) offset++;
            contents.push(look());
        }
        
        return contents;
    }

    function maybeScalar(arg = false, infix = false) {
        if (!look()) return {type: "variable", value: "_"}
        // Checks for suffixes/infixes if already the arg of another function to prevent issues
        if (arg && next() && look().type === "punctuation" && (constants.suffixes.includes(look().value) || constants.infixes.includes(look().value)) && (!infix || (infix && !constants.ninfixes.includes(look().value)))) {
            ast.contents.push(last())
            return parseFix(arg);
        } else if (arg) index--;

        if (look().type === "string" || look().type === "integer" || look().type === "boolean") {
            if (look().type === "integer") {
                if (/e[0-9]+/g.test(look().value)) {
                    return {type: "integer", value: +("1" + look().value)}
                } else {
                    return look();
                }
            } else {
                return look();
            }
        } else if (look().type === "variable") {
            let data;
            if (isPunc("{", peek())) return false;
            else if (data = isFunction(look().value)[0]) {
                let count = data[1];
                // The first argument will be passed into the function through "." if it exists.
                if (isPunc(".", last())) count -= 1;
                let args = [];
                while (count > 0) {
                    next();
                    args.push(maybeExpr());
                    count -= 1;
                }

                return {
                    type: "call",
                    value: data[0],
                    args
                };
            } else return look();
        } else {
            let save = "";
            throw new SyntaxError(`Didn't recognize token at: ${save = tokens.map(r => r.value).join("")}\n${" ".repeat(24 + index - 1) + "---^-here"}`);
        }
    }

    function parseExpr() {
        // Possible expansion in the future?
        return {
            type: "expression",
            contents: makeAST(parseContents("(", ")"))
        };
    }

    // Called from index BEFORE "{"
    function parseBlock() {
        let arg = "_"
        if (look() && look().type === "variable") arg = look().value;
        next();
        let contents = parseContents("{", "}");

        return {
            type: "block",
            arg: arg,
            contents: makeAST(contents)
        };
    }

    function parseArray() {
        return {
            type: "array",
            contents: makeAST(parseContents("[", "]"))
        };
    }

    function parseFix(arg = false) {
        let tok = look().value;
        if (arg && isPunc("\\")) {
            index--;
            return {};
        }
        if (constants.prefixes.includes(tok)) {
            // Fold
            if (tok === "\\") {
                let block = false;
                let args;
                
                if ((block = ast.contents.pop() || {}).type === "block") {
                    ast.contents.pop();
                    let fLength = getFoldLength(stream, index);
                    args = stream.slice(fLength[0], fLength[1] - (block.arg !== "_" ? 1 : 0));
                } else {
                    block = false;
                    args = stream.slice(0, stream.indexOf(look()));
                }
                next()
                return {
                    type: "prefix",
                    value: tok,
                    fold_ops: args,
                    map_ops: block,
                    arg: maybeExpr(true)
                }
            // Filter
            } else if (tok === "$") {
                if (peek().type === "variable") next();
                let contents = parseBlock();
                next();
                return {
                    type: "prefix",
                    value: tok,
                    block: contents,
                    arg: maybeExpr(true)
                };
            // Any
            } else if (tok === "$:") {
                if (peek().type === "variable") next();
                let contents = parseBlock();
                next();
                return {
                    type: "prefix",
                    value: tok,
                    block: contents,
                    arg: maybeExpr(true)
                };
            } else {
                next();
                return {
                    type: "prefix",
                    value: tok,
                    arg: maybeExpr(true)
                }
            }
        } else if (constants.infixes.includes(tok)) {
            let left;
            if (validItem(ast.contents[ast.contents.length - 1])) left = ast.contents.pop();
            next();
            if (tok === ":=") {
                while (!isPunc("(", look())) index--;
                let name = last();
                if (name.type !== "variable") throw new SyntaxError(`Incorrect function assignment at: ${save = tokens.map(r => r.value).join("")}\n${" ".repeat(24 + index - 1) + "---^-here"}`);
                let args = parseContents("(", ")");
                next(); next();

                return {
                    type: "function",
                    name,
                    args,
                    body: maybeExpr()
                };
            } else if (tok === "@") {
                if (look().type === "punctuation" && !isPunc("(") && !isPunc("[") && !isPunc("{")) {
                    next();
                    return {
                        type: "infix",
                        value: tok,
                        arg: last().value,
                        left: left || {type: "variable", value: "_"},
                        right: maybeExpr(true, true)
                    }
                } else {
                    return {
                        type: "infix",
                        value: tok,
                        left: left || {type: "variable", value: "_"},
                        right: maybeExpr(true, true)
                    }
                }
            } else {
                return {
                    type: "infix",
                    value: tok,
                    left: left || {type: "variable", value: "_"},
                    right: maybeExpr(true, true)
                };
            }
        } else if (constants.suffixes.includes(tok)) {
            let left;
            if (validItem(ast.contents[ast.contents.length - 1])) left = ast.contents.pop();
            if (tok === ":_") {
                let ops = next().value.split("");
                return {
                    type: "suffix",
                    value: tok,
                    arg: left || {type: "variable", value: "_"},
                    ops
                };
            } else {
                return {
                    type: "suffix",
                    value: tok,
                    arg: left || {type: "variable", value: "_"}
                };
            }
        } else {
            return false;
        }
    }

    function maybeExpr(arg = false, infix = false) {
        if (look() && look().type === "punctuation") {
            if (look().value === "(") {
                return parseExpr();
            } else if (look().value === "{") {
                index--;
                if (infix) return {type: "variable", value: "_"};
                return parseBlock();
            } else if (look().value === "[") {
                return parseArray();
            } else {
                return parseFix(arg);
            }
        } else {
            return maybeScalar(arg, infix);
        }
    }

    while (next()) {
        if (index >= stream.length) break;
        let obj = maybeExpr();
        if (obj) ast.contents.push(obj);
    }

    return ast;
}
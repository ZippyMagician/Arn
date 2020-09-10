const constants = require('./constants.js');
const { ArnError } = require('./errors.js');
const precedence = constants.PRECEDENCE;

function compare(original, partial) {
    return !Object.keys(partial).some((key) => partial[key] !== original[key]);
}

function getFoldLength(tokens, from) {
    let bIndex = from;
    let counter = 0;
    while (!compare(tokens[bIndex--], {type: "punctuation", value: "{"}) || counter > 0 && bIndex >= 0) {
        if (tokens[bIndex].type === "punctuation" && tokens[bIndex].value === "}") counter++;
        else if (tokens[bIndex].type === "punctuation" && tokens[bIndex].value === "{") counter--;
    };
    bIndex += 1;
    return [0, bIndex];
}

// Version 1 as of 8/6/2020 2:42 PM EST
module.exports.makeAST = function makeAST(tokens, original, parent_ast = false) {
    const stream = tokens;
    let index = -1;
    const last = () => stream[index - 1];
    const look = () => stream[index];
    const peek = () => stream[index + 1];
    const next = () => stream[++index];

    let ast = {type: "prog", contents: []};
    // Stores precedence info
    let current_prec = false;

    function isPunc(char, val = false) {
        return (val || look()) && (val || look()).type === "punctuation" && (val || look()).value === char;
    }

    // TODO: Make sure this will work for all edge cases
    function validItem(obj) {
        return obj && obj.type !== "block" && obj.type !== "function" && (precedence[obj.value] && obj.type !== "string" ? precedence[obj.value] >= current_prec : true);// && obj.type !== "infix";
    }

    function isFunction(key) {
        let mapped = ast.contents.filter(r => r.type === "function").map(r => [r.value, r.args && r.args.length]);
        for (let key in constants.builtins) mapped.push([key, constants.builtins[key]]);
        if (parent_ast) mapped = mapped.concat(parent_ast.contents.filter(r => r.type === "function").map(r => [r.value, r.args && r.args.length]));
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
        if (arg && next() && look().type === "punctuation" && precedence[look().value] && precedence[look().value] > current_prec) {
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
                let current = look();
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
                    args,
                    pos: current.pos,
                    line: current.line
                };
            } else if (isPunc(":=", peek()) || (isPunc("(", peek()) && stream.filter(r => isPunc(':=', r)))) {
                // The creation of a function
                let name = look();
                next();
                let args;
                let current;
                if (isPunc("(")) {
                    current = look();
                    args = parseContents("(", ")");
                    next();
                }

                if (!isPunc(":=", look())) {
                    ast.contents.push(name);
                    index--;
                    if (args && args.length) return {
                        type: "expression",
                        contents: makeAST(args, original, parent_ast || ast),
                        pos: current.pos,
                        line: current.line
                    }; else return {};
                } else {
                    next();
                    return {
                        type: "function",
                        value: name.value,
                        args: args || false,
                        body: maybeExpr(),
                        pos: name.pos,
                        line: name.line
                    };
                }
            } else return look();
        } else {
            let save = "";
            throw ArnError("Couldn't recognize token", original, look().line, look().pos);
        }
    }

    function parseExpr() {
        let current = look();
        let obj = {
            type: "expression",
            contents: makeAST(parseContents("(", ")"), original, parent_ast || ast),
            pos: current.pos,
            line: current.line
        };

        if (next() && look().type === "punctuation" && precedence[look().value] && precedence[look().value] > current_prec) {
            ast.contents.push(obj);
            return parseFix(true);
        } else {
            index--;
            return obj;
        }
    }

    // Called from index BEFORE "{"
    function parseBlock() {
        let arg = "_"
        if (look() && look().type === "variable") arg = look().value;
        next();
        let current = look();
        let contents = parseContents("{", "}");

        return {
            type: "block",
            arg: arg,
            contents: makeAST(contents, original, parent_ast || ast),
            pos: current.pos,
            line: current.line
        };
    }

    function parseArray() {
        let current = look();
        let obj = {
            type: "array",
            contents: makeAST(parseContents("[", "]"), original, parent_ast || ast),
            pos: current.pos,
            line: current.line
        };

        if (next() && look().type === "punctuation" && precedence[look().value] && precedence[look().value] > current_prec) {
            ast.contents.push(obj);
            return parseFix(true);
        } else {
            index--;
            return obj;
        }
    }

    function parseFix(arg = false) {
        let tok = look().value;
        let current = look();
        let ret_obj;

        if (arg && isPunc("\\")) {
            index--;
            return false;
        }
        current_prec = precedence[tok];
        if (constants.prefixes.includes(tok)) {
            // Fold
            if (tok === "\\") {
                let block = false;
                let args;
                
                if ((block = ast.contents.pop() || {}).type === "block") {
                    ast.contents.pop();
                    let fLength = getFoldLength(stream, index);
                    if (fLength[1] === 0) args = [];
                    else args = stream.slice(fLength[0], fLength[1] - (block.arg !== "_" ? 1 : 0));
                } else {
                    block = false;
                    args = stream.slice(0, stream.indexOf(look()));
                }
                next();
                ret_obj = {
                    type: "prefix",
                    value: tok,
                    fold_ops: args,
                    map_ops: block,
                    arg: maybeExpr(true) || {type: "variable", value: "_"},
                    pos: current.pos,
                    line: current.line
                }
            // Filter
            } else if (tok === "$") {
                if (peek().type === "variable") next();
                let contents = parseBlock();
                if (!contents) throw ArnError("Must provide block to prefix $", original, current.line, current.pos);
                next();
                ret_obj = {
                    type: "prefix",
                    value: tok,
                    block: contents,
                    arg: maybeExpr(true) || {type: "variable", value: "_"},
                    pos: current.pos,
                    line: current.line
                };
            // Any
            } else if (tok === "$:") {
                if (peek().type === "variable") next();
                let contents = parseBlock();
                if (!contents) throw ArnError("Must provide block to prefix $:", original, current.line, current.pos);
                next();
                ret_obj = {
                    type: "prefix",
                    value: tok,
                    block: contents,
                    arg: maybeExpr(true) || {type: "variable", value: "_"},
                    pos: current.pos,
                    line: current.line
                };
            } else {
                next();
                ret_obj = {
                    type: "prefix",
                    value: tok,
                    arg: maybeExpr(true) || {type: "variable", value: "_"},
                    pos: current.pos,
                    line: current.line
                }
            }
        } else if (constants.infixes.includes(tok)) {
            let left;
            if (validItem(ast.contents[ast.contents.length - 1])) left = ast.contents.pop();
            next();
            if (tok === "." && (!look() || look().type !== "variable")) throw ArnError("Cannot call dot infix on a non-function.", original, current.line, current.pos);
            if (tok === "@") {
                if (look().type === "punctuation" && !isPunc("(") && !isPunc("[") && !isPunc("{")) {
                    next();
                    ret_obj = {
                        type: "infix",
                        value: tok,
                        arg: last(),
                        left: left || {type: "variable", value: "_"},
                        right: maybeExpr(true, true) || {type: "variable", value: "_"},
                        pos: current.pos,
                        line: current.line
                    }
                } else {
                    ret_obj = {
                        type: "infix",
                        value: tok,
                        left: left || {type: "variable", value: "_"},
                        right: maybeExpr(true, true) || {type: "variable", value: "_"},
                        pos: current.pos,
                        line: current.line
                    }
                }
            } else {
                ret_obj = {
                    type: "infix",
                    value: tok,
                    left: left || {type: "variable", value: "_"},
                    right: maybeExpr(true, true) || {type: "variable", value: "_"},
                    pos: current.pos,
                    line: current.line
                };
            }
        } else if (constants.suffixes.includes(tok)) {
            let left;
            if (validItem(ast.contents[ast.contents.length - 1])) left = ast.contents.pop();
            if (tok === ";") {
                let ops = next().value.split("");
                ret_obj = {
                    type: "suffix",
                    value: ";",
                    arg: left || {type: "variable", value: "_"},
                    ops,
                    pos: current.pos,
                    line: current.line
                };
            } else {
                ret_obj = {
                    type: "suffix",
                    value: tok,
                    arg: left || {type: "variable", value: "_"},
                    pos: current.pos,
                    line: current.line
                };
            }
        } else {
            return false;
        }
        if (ret_obj && next() && precedence[look().value] && current_prec <= precedence[look().value]) {
            ast.contents.push(ret_obj);
            ast.contents.push(parseFix(true));
            return ast.contents.pop();
        } else {
            index--;
            return ret_obj;
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
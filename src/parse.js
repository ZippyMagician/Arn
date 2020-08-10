const Environment = require('./env.js');
const ast = require('./ast.js');
const tokenize = require('./lexer.js');
const dictionary = require('./dictionary');
const rl = require('readline-sync');

var stdin = false;

function cast(value, type) {
    switch (type) {
        case "int":
            return typeof value === "array" ? value[0] : +value;
        case "string":
            return typeof value === "string" ? value : typeof value === "number" ? value.toString() : value[0];
        case "array":
            return typeof value === "string" || typeof value === "number" ? value.toString().split(value.toString().indexOf(" ") > -1 ? " " : "") : value;
    }
}

module.exports = (tree, opts) => {
    function zip(left, right) {
        return left.map((val, index) => [val, right[index]]);
    }
    
    function zip_with(left, right, op, env) {
        return zip(left, right).map(entry => evalNode(entry.join(op.value), env));
    }

    // Overhead for all the punctuation
    function evalPrefix(node, env) {
        const coerce = (n, t) => cast(evalNode(n.arg, env), t);
        let func;
        let ind;
    
        switch (node.value) {
            case 'n':
                return -1 * coerce(node, "int");
            case '!!':
                return [...coerce(node, "string")].reverse().join("");
            case '!':
                return !evalNode(node.arg, env);
            case ':v':
                return Math.floor(coerce(node, "int"));
            case ':^':
                return Math.ceil(coerce(node, "int"));
            case '++':
                return ++coerce(node, "int");
            case '--':
                return --coerce(node, "int");
            case ':*':
                return coerce(node, "int") ** 2;
            case ':/':
                return Math.sqrt(coerce(node, "int"));
            case ':>':
                return coerce(node, "array").sort();
            case ':<':
                return coerce(node, "array").sort((a, b) => b - a);
            case '$':
                func = node.block.contents;
                ind = node.block.arg;
    
                const filter = v => {
                    let child_env = env.clone();
                    child_env.set(ind, {type: "string", value: v});
                    let ret_val = evalNode(func, child_env);
                    env.update(child_env, ind);

                    return ret_val;
                }
    
                return coerce(node, "array").filter(filter);
            case '$:':
                func = node.block.contents;
                ind = node.block.arg;
    
                const any_filter = v => {
                    let child_env = env.clone();
                    child_env.set(ind, {type: "string", value: v});
                    let ret_val = evalNode(func, child_env);
                    env.update(child_env, ind);

                    return ret_val;
                }
    
                return coerce(node, "array").filter(any_filter).length > 0;
            case '|:':
                let scalar = coerce(node, "string");
    
                return [scalar, [...scalar].reverse().join("")];
            case '\\':
                let map_ops = node.map_ops;
                let fold_ops = node.fold_ops;
    
                let val = coerce(node, "array");
    
                if (map_ops) {
                    const prefix_map = v => {
                        let child_env = env.clone();
                        child_env.set(map_ops.block.arg, {type: "string", value: v});
                        let ret = evalNode(map_ops.block.contents, child_env);
                        env.update(child_env, map_ops.block.arg);

                        return ret;
                    }
                    
                    val = val.map(prefix_map);
                }
                
                if (fold_ops.length) return evalNode(ast(tokenize(val.join(` ${fold_ops.map(r => r.value).join("")} `))), env);
                else return val;
            default:
                throw new SyntaxError("Couldn't recognize prefix: " + node.value);
        }
    }
    
    function evalInfix(node, env) {
        const coerce = (n, t) => cast(evalNode(n, env), t);
        
        switch (node.value) {
            case '=':
                return evalNode(node.left, env) == evalNode(node.right, env);
            case '<':
                return evalNode(node.left, env) < evalNode(node.right, env);
            case '>':
                return evalNode(node.left, env) > evalNode(node.right, env);
            case '<=':
                return evalNode(node.left, env) <= evalNode(node.right, env);
            case '>=':
                return evalNode(node.left, env) >= evalNode(node.right, env);
            case '!=':
                return evalNode(node.left, env) != evalNode(node.right, env);
            case '||':
                return evalNode(node.left, env) || evalNode(node.right, env);
            case '&&':
                return evalNode(node.left, env) && evalNode(node.right, env);
            case '+':
                return coerce(node.left, "int") + coerce(node.right, "int");
            case '-':
                return coerce(node.left, "int") - coerce(node.right, "int");
            case '*':
                return coerce(node.left, "int") * coerce(node.right, "int");
            case '/':
                return coerce(node.left, "int") / coerce(node.right, "int");
            case '%':
                return coerce(node.left, "int") % coerce(node.right, "int");
            case '^':
                return coerce(node.left, "int") ** coerce(node.right, "int");
            case '|':
                return coerce(node.left, "string") + coerce(node.right, "string");
            case '@':
                if (node.arg) {
                    return zip_with(evalNode(node.left, env), evalNode(node.right, env), node.arg, env);
                } else {
                    return zip(evalNode(node.left, env), evalNode(node.right, env));
                }
            case ':|':
                return coerce(node.left, "array").join(coerce(node.right, "string"));
            case '->':
            case '=>':
                let range = [];
                let ind = coerce(node.left, "int");
                let end = coerce(node.right, "int");
    
                if (node.value === "->") for (ind; ind < end; ind++) range.push(ind);
                else for (ind; ind <= end; ind++) range.push(ind);
    
                return range;
            case ':!':
                return coerce(node.left, "string").split(coerce(node.right, "string"));
            case '.':
                node.right.args = [env.get(node.left.value), ...node.right.args];
                return evalNode(node.right, env);
            case '&':
                return coerce(node.left, "array").indexOf(coerce(node.right, "string")) > -1;
            case '@:':
                let left = node.left;
                if (left.type !== "keyword") throw new SyntaxError("Cannot modify immutable item:", left);

                let obj = env.get(left.value);
                let entry = coerce(obj, "array");
                let index = entry.indexOf(coerce(node.right, "string"));

                env.set(left.value, {type: "array", contents: {type: "prog", contents: [...entry.slice(0, index), ...entry.slice(index + 1)].map(r => {return {type: "string", value: r}})}});

                return evalNode(env.get(left.value), env);
            case '?':
                return coerce(node.left, "array")[coerce(node.right, "int")];
            default:
                throw new SyntaxError("Could not recognize infix:", node);
        }
    }
    
    function doBase(command, ops, item, length) {
        if (!command) return item;
        switch (command) {
            case 'b':
                return (+item).toString(2).padStart(length, '0');
            case 'h':
                return (+item).toString(16).padStart(length, '0');
            case 'o':
                return (+item).toString(8).padStart(length, '0');
            case 'd':
                return (+item);
            case 'O':
                return doBase(ops[1], ops, parseInt(item, 8), length);
            case 'H':
                return doBase(ops[1], ops, parseInt(item, 16), length);
            case 'B':
                return doBase(ops[1], ops, parseInt(item, 2), length);
            default:
                throw new SyntaxError("Issue with base parsing:", command, ops, item);
        }
    }
    
    function evalSuffix(node, env) {
        const coerce = (n, t) => cast(evalNode(n.arg, env), t);
        
        switch (node.value) {
            case '#':
                return evalNode(node.arg, env).length;
            case ':_':
                let ops = node.ops[0].split("");
                let length = 0;
                let command;
    
                if (/[0-9]/g.test(ops[0])) {
                    length = +ops[0];
                    command = ops[1];
                    ops.shift();
                } else {
                    command = ops[0];
                }
    
                return doBase(command, ops, coerce(node, "string"), length);
            case '^*':
                return coerce(node, "int") > 0 && Math.sqrt(coerce(node, "int")) % 1 === 0;
            case ':n':
                return coerce(node, "string").split("\n");
            case ':s':
                return coerce(node, "string").split(" ");
        }
    }

    var env = new Environment();
    // Evaluates current node of tree
    function evalNode(node, env) {
        let ret_val = "";

        switch (node.type) {
            case "prog":
                for (let child_node of node.contents) {
                    ret_val = evalNode(child_node, env);
                }
                break;
            case "string":
                if (node.char) {
                    ret_val = dictionary.decompress(node.value, node.char === `'`)
                } else {
                    ret_val = node.value;
                }
                break;
            case "integer":
                ret_val = +node.value;
                break;
            case "expression":
                ret_val = evalNode(node.contents, env);
                break;
            case "array":
                ret_val = [];
                for (let child_node of node.contents.contents) {
                    ret_val.push(evalNode(child_node, env));
                }
                break;
            case "function":
                env.create_func(node.name, node.args, node.body);
                break;
            case "call":
                let [arg_list, body] = env.get_func(node.value);
                if (arg_list.filter(r => r.type !== "keyword").length > 0) throw new SyntaxError("Cannot pass non-keywords as argument names to function: " + node.value);
                let child_env = env.clone();

                for (let i in arg_list) {
                    child_env.set(arg_list[i].value, node.args[i]);
                }
                
                ret_val = evalNode(body, child_env);
                env.update(child_env);
                break;
            case "prefix":
                ret_val = evalPrefix(node, env);
                break;
            case "infix":
                ret_val = evalInfix(node, env);
                break;
            case "suffix":
                ret_val = evalSuffix(node, env);
                break;
            case "keyword":
                ret_val = evalNode(env.get(node.value), env);
                break;
            case "javascript":
                ret_val = eval(`${env.storage.map(obj => `var ${obj.name} = ${evalNode(obj.value, env) || '""'}`).join(";")};${node.body}`);
                break;
            default:
                throw new SyntaxError("Unrecognized node in AST:", node);
        }

        return ret_val;
    }
    
    if (opts.stdin) stdin = opts.stdin.toString().indexOf("\n") > -1 ? opts.stdin.toString().split("\n") : [opts.stdin.toString()];

    function define_func(name, args, fn) {
        env.create_func(name, args, ast(tokenize(fn)));
    }

    function hardcode(name, args, fn) {
        env.create_func(name, args, {type: "javascript", body: fn});
    }

    // Create constant values in environment
    env.set("f", {type: "integer", value: 0});
    env.set("t", {type: "integer", value: 1});
    env.set("c", {type: "string", value: ""});
    env.set("pi", {type: "integer", value: "3.14159265358979323846"});
    env.set("e", {type: "integer", value: "2.71828182845904523536"});
    env.set("_", {
        type: "array",
        contents: {
            type: "prog", 
            contents: stdin ? stdin.map(str => {return {
                type: "string",
                value: str
            }}) : [{type: "string", value: rl.question("STDIN: ")}]
        }
    });

    let item;
    if ((item = env.get("_").contents.contents).length === 1) env.set("_", item[0]);

    let std = [{type: "keyword", value: "_"}];

    define_func("max", std, ":<(?0");
    define_func("min", std, ":>(?0");
    hardcode("out", std, "process.stdout.write(_ + \" \");");
    hardcode("in", std, "stdin || rl.question('> ')");
    define_func("outl", std, "out |\"\n\"");
    define_func("intr", std.concat([{type: "keyword", value: "sep"}]), "|{|sep}\\ :! c");
    define_func("fact", std, "*\\ 1=>");
    define_func("mean", std, "(+\\) / #");

    return evalNode(tree, env);
}
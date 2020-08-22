const Environment = require('./env.js');
const ast = require('./ast.js');
const tokenize = require('./lexer.js');
const dictionary = require('./dictionary');
const rl = require('readline-sync');

const { cast, printf, stringify } = require('./formatter.js');
const Sequence = require('./sequence.js');
const { default: BigNumber } = require('bignumber.js');

var stdin = false;

module.exports = (tree, opts) => {
    function zip(left, right) {
        return left.map((val, index) => [val, right[index]]);
    }
    
    function zip_with(left, right, op, env) {
        return zip(left, right).map(entry => evalNode(ast(tokenize(entry.map(r => stringify(r)).join(` ${op.value} `))), env));
    }

    // Overhead for all the punctuation
    function evalPrefix(node, env, f = false) {
        const coerce = (n, t) => cast(evalNode(n.arg, env), t);
        const fix = item => /^\d+$/.test(item) ? +item : item;
        const unpack = (n) => n instanceof BigNumber ? fix(n.toString()) : n;

        let func;
        let ind;
        let value;
    
        switch (node.value) {
            case 'n_':
                return coerce(node, "int").multipliedBy(new BigNumber(-1)).toString();
            case '!!':
                return [...coerce(node, "string")].reverse().join("");
            case '!':
                const fix = item => /^\d+$/.test(item) ? +item : item;
                return !fix(evalNode(node.arg, env, true)) ? 1 : 0;
            case ':v':
                return Math.floor(fix(evalNode(node.arg, env, true))).toString();
            case ':^':
                return Math.ceil(fix(evalNode(node.arg, env, true))).toString();
            case '++':
                if (node.arg.type === "variable") {
                    value = evalNode(env.get(node.arg.value), env, true);
                    if (typeof value === "object") {
                        env.set(node.arg.value, {type: "array", contents: {type: "prog", contents: value.map(r => {return {type: "integer", value: ++r}})}});
                        return value.map(r => ++r);
                    } else {
                        env.set(node.arg.value, {type: "integer", value: ++value});
                        return value;
                    }
                } else {
                    return coerce(node, "int").plus(new BigNumber(1)).toString();
                }
            case '--':
                if (node.arg.type === "variable") {
                    value = fix(evalNode(env.get(node.arg.value), env, true));
                    if (typeof value === "object") {
                        env.set(node.arg.value, {type: "array", contents: {type: "prog", contents: value.map(r => {return {type: "integer", value: --r}})}});
                        return value.map(r => --r);
                    } else {
                        env.set(node.arg.value, {type: "integer", value: --value});
                        return value;
                    }
                } else {
                    return coerce(node, "int").minus(new BigNumber(1)).toString();
                }
            case ':*':
                return coerce(node, "int").exponentiatedBy(new BigNumber(2)).toString();
            case ':/':
                return coerce(node, "int").squareRoot().toString();
            case ':>':
                return coerce(node, "array").map(r => unpack(r)).sort((a, b) => (typeof a === "object" ? a.length : a) - (typeof b === "object" ? b.length : b));
            case ':<':
                return coerce(node, "array").map(r => unpack(r)).sort((a, b) => (typeof b === "object" ? b.length : b) - (typeof a === "object" ? a.length : a));
            case '$':
                func = node.block.contents;
                ind = node.block.arg;
    
                const filter = v => {
                    let child_env = env.clone();
                    child_env.set(ind, {type: "string", value: v});
                    let ret_val = evalNode(func, child_env, true);
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
                    let ret_val = evalNode(func, child_env, true);
                    env.update(child_env, ind, true);

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
                        child_env.set(map_ops.arg, {type: "string", value: v});
                        let ret = evalNode(map_ops.contents, child_env, true);
                        env.update(child_env, map_ops.arg);

                        return ret;
                    }
                    
                    val = val.map(prefix_map);
                }
                
                // Values won't parse properly unless they are stringified to represent their actual type in the data
                if (fold_ops.length) {
                    if (val.length == 1) return val[0];
                    val = val.map(r => stringify(r));
                    console.log(val.join(` ${fold_ops.map(r => r.value).join("")} `))
                    return evalNode(ast(tokenize(val.join(` ${fold_ops.map(r => r.value).join("")} `))), env, true);
                }
                else return val;
            default:
                throw new SyntaxError("Couldn't recognize prefix: " + node.value);
        }
    }
    
    function evalInfix(node, env, f = false) {
        const coerce = (n, t) => cast(evalNode(n, env), t);
        const fix = item => /^\d+$/.test(item) ? +item : item;
        
        switch (node.value) {
            case '=':
                return evalNode(node.left, env, true) == evalNode(node.right, env, true);
            case '<':
                return coerce(node.left, "int").isLessThan(coerce(node.right, "int"));
            case '>':
                return coerce(node.left, "int").isGreaterThan(coerce(node.right, "int"));
            case '<=':
                return coerce(node.left, "int").isLessThanOrEqualTo(coerce(node.right, "int"));
            case '>=':
                return coerce(node.left, "int").isGreaterThanOrEqualTo(coerce(node.right, "int"));
            case '!=':
                return evalNode(node.left, env, true) != evalNode(node.right, env, true);
            case '||':
                return fix(evalNode(node.left, env, true)) || fix(evalNode(node.right, env, true));
            case '&&':
                return fix(evalNode(node.left, env, true)) && fix(evalNode(node.right, env, true));
            case '+':
                return coerce(node.left, "int").plus(coerce(node.right, "int")).toString();
            case '-':
                return coerce(node.left, "int").minus(coerce(node.right, "int")).toString();
            case '*':
                return coerce(node.left, "int").multipliedBy(coerce(node.right, "int")).toString();
            case '/':
                return coerce(node.left, "int").dividedBy(coerce(node.right, "int")).toString();
            case '%':
                let mod_right = coerce(node.right, "int");
                // Javascript has a f*cking annoying modulo bug for negatives
                return coerce(node.left, "int").modulo(mod_right).plus(mod_right).modulo(mod_right).toString();
            case '^':
                let repeat;
                if (typeof (repeat = evalNode(node.left, env)) === "string") {
                    return repeat.repeat(coerce(node.right, "int").toString());
                } else {
                    return coerce(node.left, "int").exponentiatedBy(coerce(node.right, "int")).toString();
                }
            case '|':
                return coerce(node.left, "string") + coerce(node.right, "string");
            case '@':
                if (node.arg) {
                    return zip_with(coerce(node.left, "array"), coerce(node.right, "array"), node.arg, env);
                } else {
                    return zip(coerce(node.left, "array"), coerce(node.right, "array"));
                }
            case ':|':
                return coerce(node.left, "array").join(coerce(node.right, "string"));
            case ':i':
                return coerce(node.left, "array", true).indexOf(evalNode(node.right, env, true));
            case '->':
            case '=>':
                let range = [];
                let ind = coerce(node.left, "int").toNumber();
                let end = coerce(node.right, "int").toNumber();
    
                if (node.value === "->") for (ind; ind < end; ind++) range.push(ind);
                else for (ind; ind <= end; ind++) range.push(ind);
                return range;
            case ':!':
                return coerce(node.left, "string").split(coerce(node.right, "string"));
            case '.':
                node.right.args = [node.left, ...node.right.args];
                return evalNode(node.right, env);
            case '&':
                return coerce(node.left, "array").indexOf(coerce(node.right, "string")) > -1;
            case '@:':
                let left = node.left;
                if (left.type !== "variable") throw new SyntaxError("Cannot modify immutable item:", left);

                let obj = env.get(left.value);
                let entry = coerce(obj, "array");
                let index = entry.indexOf(coerce(node.right, "string"));

                env.set(left.value, {type: "array", contents: {type: "prog", contents: [...entry.slice(0, index), ...entry.slice(index + 1)].map(r => {return {type: "string", value: r}})}});

                return evalNode(env.get(left.value), env);
            case '?':
                let arr = coerce(node.left, "array");
                if (arr.get) return arr.get(coerce(node.right, "int"));
                else return arr[coerce(node.right, "int")];
            default:
                throw new SyntaxError("Could not recognize infix:", node);
        }
    }
    
    function doBase(command, ops, item, length) {
        if (!command) return item;
        switch (command) {
            case 'b':
                return item.toNumber().toString(2).padStart(length, '0');
            case 'h':
                return item.toNumber().toString(16).padStart(length, '0');
            case 'o':
                return item.toNumber().toString(8).padStart(length, '0');
            case 'd':
                return item.toNumber();
            case 'O':
                return doBase(ops[1], ops, new BigNumber(item.toNumber(), 8), length);
            case 'H':
                return doBase(ops[1], ops, new BigNumber(item.toNumber(), 16), length);
            case 'B':
                return doBase(ops[1], ops, new BigNumber(item.toNumber(), 2), length);
            default:
                throw new SyntaxError("Issue with base parsing:", command, ops, item);
        }
    }
    
    function evalSuffix(node, env) {
        const coerce = (n, t) => cast(evalNode(n.arg, env), t);
        
        switch (node.value) {
            case '#':
                return evalNode(node.arg, env, true).length;
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
    
                return doBase(command, ops, coerce(node, "int"), length);
            case '^*':
                return coerce(node, "int") > 0 && Math.sqrt(coerce(node, "int")) % 1 === 0;
            case ':n':
                return coerce(node, "string").split("\n");
            case ':s':
                return coerce(node, "string").split(" ");
            case ':{':
                return coerce(node, "array")[0];
            case ':}':
                let item = coerce(node, "array");
                return item[item.length - 1];
            case ':@':
                let arr = coerce(node, "array");
                // Splice first element so reduce will work properly
                arr = [arr[0], ...arr];
                    
                return arr.reduce((acc, val) => typeof acc === "object" ? (acc.filter(entry => entry[0] === val).length ? (acc[acc.indexOf(acc.filter(entry => entry[0] === val)[0])].push(val), acc) : (acc.push([val]), acc)) : [[val]]);
            default:
                throw new SyntaxError("Couldn't recognize suffix: " + node.value);
        }
    }

    var env = new Environment();
    // Evaluates current node of tree
    function evalNode(node, env, fix = false) {
        let ret_val = "";
        let child_env

        switch (node.type) {
            case "prog":
                for (let child_node of node.contents) {
                    ret_val = evalNode(child_node, env, fix);
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
                ret_val = new BigNumber(node.value);
                if (fix) ret_val = ret_val.toString();
                break;
            case "expression":
                ret_val = evalNode(node.contents, env, fix);
                break;
            case "block":
                child_env = env.clone();
                child_env.set(node.arg, env.get("_"));

                ret_val = evalNode(node.contents, child_env, fix);
                env.update(child_env, node.arg);
                break;
            case "array":
                // Check if sequence
                if (node.contents.contents.filter(r => r.type === "block").length) {
                    let container;
                    let seq_length = (container = node.contents.contents.filter(r => r.type === "infix" && r.value === "->")).length ? evalNode(container[0].right, env, fix) : false;
                    let seq_block = node.contents.contents.filter(r => r.type === "block")[0];

                    ret_val = new Sequence(node.contents.contents.filter(r => r.type !== "block" && !(r.type === "infix" && r.value === "->")).map(r => evalNode(r, env, fix)), seq_block, seq_length, env, evalNode);
                } else {
                    ret_val = [];
                    for (let child_node of node.contents.contents) {
                        ret_val.push(evalNode(child_node, env, fix));
                    }
                }
                break;
            case "function":
                env.create_func(node.name, node.args, node.body);
                break;
            case "call":
                let [arg_list, body] = env.get_func(node.value);
                if (arg_list.filter(r => r.type !== "variable").length > 0) throw new SyntaxError("Cannot pass non-variables as argument names to function: " + node.value);
                child_env = env.clone();

                for (let i in arg_list) {
                    child_env.set(arg_list[i].value, node.args[i]);
                }
                
                ret_val = evalNode(body, child_env, fix);
                env.update(child_env);
                break;
            case "prefix":
                ret_val = evalPrefix(node, env, fix);
                break;
            case "infix":
                ret_val = evalInfix(node, env, fix);
                break;
            case "suffix":
                ret_val = evalSuffix(node, env, fix);
                break;
            case "variable":
                ret_val = evalNode(env.get(node.value), env, fix);
                break;
            case "javascript":
                env.get(node.name).body(env);
                break;
            default:
                throw new SyntaxError("Unrecognized node in AST:", node);
        }

        return ret_val;
    }
    
    if (opts.stdin) stdin = opts.stdin.toString().indexOf("\\n") > -1 ? opts.stdin.toString().split("\\n") : [opts.stdin.toString()];

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
    env.set("pi", {type: "integer", value: "3.14159265358979323846264338327950288"});
    env.set("e", {type: "integer", value: "2.71828182845904523536028747135266249"});
    env.set("phi", {type: "integer", value: "1.61803398874989484820458683436563811"})
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

    let std = [{type: "variable", value: "_"}];

    define_func("max", std, "(:<):{");
    define_func("min", std, "(:>):{");
    hardcode("out", std, (env) => printf(env.get("_")));
    hardcode("in", [], (env) => stdin || rl.question("> "));
    define_func("outl", std, "out |\"\n\"");
    define_func("intr", std.concat([{type: "variable", value: "sep"}]), "|\\ (@| sep)");
    define_func("fact", std, "*\\ 1=>");
    define_func("mean", std, "(+\\) / #");
    define_func("mode", std, ":< :@ :{:{");

    return evalNode(tree, env);
}
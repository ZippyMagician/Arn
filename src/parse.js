const { Environment } = require('./env.js');
const { makeAST: ast } = require('./ast.js');
const { tokenize } = require('./lexer.js');
const dictionary = require('./dictionary');
const rl = require('readline-sync');

const { cast, printf, stringify, constructType } = require('./formatter.js');
const { ArnError } = require('./errors.js');
const { Sequence } = require('./sequence.js');
const { default: BigNumber } = require('bignumber.js');

const math = require('./math.js');

var stdin = false;
BigNumber.set({
    ALPHABET: '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ',
    DECIMAL_PLACES: 25
});

module.exports.walkTree = function parse(tree, opts, original) {
    if (opts.p || opts.precision) BigNumber.set({DECIMAL_PLACES: opts.precision || opts.p});
    
    function zip(...vals) {
        return vals[0].map((_, i) => vals.map(array => array[i]));
    }
    
    function zip_with(left, right, op, env) {
        return zip(left, right).map(entry => evalNode(ast(tokenize(entry.map(r => stringify(r)).join(` ${op.value} `)), original), env));
    }

    // Overhead for all the punctuation
    function evalPrefix(node, env, f = false) {
        const coerce = (n, t, f = false) => cast(evalNode(n.arg, env, f), t);
        const fix = item => /^\d+$/.test(item) ? +item : item;
        const unpack = (n) => n instanceof BigNumber ? fix(n.toString()) : n;

        let func;
        var ind;
        let value;
    
        switch (node.value) {
            case 'n_':
                return coerce(node, "int").multipliedBy(new BigNumber(-1)).toString();
            case '!!':
                return [...coerce(node, "string")].reverse().join("");
            case '!':
                return !fix(evalNode(node.arg, env, true)) ? 1 : 0;
            case ':v':
                return Math.floor(fix(evalNode(node.arg, env, true))).toString();
            case ':^':
                return Math.ceil(fix(evalNode(node.arg, env, true))).toString();
            case '++':
                if (node.arg.type === "variable") {
                    value = evalNode(env.get(node.arg.value, node.line, node.pos), env, true);
                    if (typeof value === "object") {
                        env.set(node.arg.value, {type: "array", contents: {type: "prog", contents: value.map(r => {return {type: "integer", value: ++r}})}});
                        return value.map(r => ++r);
                    } else {
                        env.set(node.arg.value, {type: "integer", value: ++value});
                        return value;
                    }
                } else {
                    return coerce(node, "int").plus(1).toString();
                }
            case '--':
                if (node.arg.type === "variable") {
                    value = fix(evalNode(env.get(node.arg.value, node.line, node.pos), env, true));
                    if (typeof value === "object") {
                        env.set(node.arg.value, {type: "array", contents: {type: "prog", contents: value.map(r => {return {type: "integer", value: --r}})}});
                        return value.map(r => --r);
                    } else {
                        env.set(node.arg.value, {type: "integer", value: --value});
                        return value;
                    }
                } else {
                    return coerce(node, "int").minus(1).toString();
                }
            case ':*':
                return coerce(node, "int").exponentiatedBy(2).toString();
            case ':/':
                return coerce(node, "int").squareRoot().toString();
            case ':+':
                return coerce(node, "int").multipliedBy(2).toString();
            case ':-':
                return coerce(node, "int").dividedBy(2).toString();
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
    
                let val = coerce(node, "array", true);
                
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
                    let repair_negatives = n => n.type === "integer" && +n.value < 0 ? n.value.replace(/\-/g, "(n_") + ")" : n.value;
                    if (val.length == 1) return val[0];
                    val = val.map(r => stringify(r));
                    
                    return evalNode(ast(tokenize(val.join(` ${fold_ops.map(r => repair_negatives(r)).join("")} `)), original), env, true);
                }
                else return val;
            case '~':
                let range = [];
                ind = 1;
                let end = coerce(node, "int").toNumber();
    
                for (ind; ind <= end; ind++) range.push(ind);
                return range;
            case '?.':
                let vec = coerce(node, "array");
                return unpack(vec[Math.floor(Math.random() * vec.length)]);
            case '#.':
                return math.listPrimes(unpack(coerce(node, "int")));
            case '*.':
                return math.factorize(unpack(coerce(node, "int")));
            default:
                throw ArnError("Couldn't recognize prefix.", original, node.line, node.pos);
        }
    }
    
    function evalInfix(node, env, f = false) {
        const coerce = (n, t, c = false) => cast(evalNode(n, env, c), t);
        const fix = item => /^\d+$/.test(item) ? +item : item;
        
        switch (node.value) {
            case ':':
                let varName = node.left.value;
                let varValue = evalNode(node.right, env, true);

                env.set(varName, constructType(varValue));
                return varValue;
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
                let arr_test = coerce(node.left, "array", true);
                return arr_test.indexOf(coerce(node.right, "string")) > -1 || arr_test.indexOf(coerce(node.right, "int").toNumber()) > -1;
            case '@:':
                let left = node.left;
                if (left.type !== "variable") throw ArnError("Cannot modify immutable value.", original, left.line, left.pos);
                let entry = coerce(node.left, "array", true);
                let index = entry.indexOf(evalNode(node.right, env, true));

                env.set(left.value, {type: "array", contents: {type: "prog", contents: [...entry.slice(0, index), ...entry.slice(index + 1)].map(r => {return {type: "string", value: r}})}});

                return evalNode(env.get(left.value, left.line, left.pos), env);
            case '?':
                let arr = coerce(node.left, "array");
                if (arr.get) return arr.get(coerce(node.right, "int"));
                else return arr[coerce(node.right, "int")];
            case ',':
                return [evalNode(node.left, env, true), evalNode(node.right, env, true)];
            default:
                throw ArnError("Couldn't recognize infix.", original, node.line, node.pos);
        }
    }
    
    function doBase(command, ops, item, length, node) {
        if (!command) return item;
        switch (command) {
            case 'b':
                return item.toString(2).padStart(length, '0');
            case 'h':
                return item.toString(16).padStart(length, '0');
            case 'o':
                return item.toString(8).padStart(length, '0');
            case 'd':
                return item.toString(10);
            case 'O':
                return doBase(ops[1], ops.slice(1), new BigNumber(item.toString(10), 8), length, node);
            case 'H':
                return doBase(ops[1], ops.slice(1), new BigNumber(item.toString(10), 16), length, node);
            case 'B':
                return doBase(ops[1], ops.slice(1), new BigNumber(item.toString(10), 2), length, node);
            default:
                throw ArnError("Invalid base conversion.", original, node.line, node.pos);
        }
    }
    
    function evalSuffix(node, env) {
        const coerce = (n, t, f = false) => cast(evalNode(n.arg, env, f), t);
        
        switch (node.value) {
            case ';':
                let ops = [ ...node.ops ];
                let length = 0;
                let command;
        
                if (/[0-9]/g.test(ops[0])) {
                    length = +ops.shift();
                    command = ops
                }
                command = ops[0];
                return doBase(command, ops, coerce(node, "int"), length, node);
            case '#':
                return evalNode(node.arg, env, true).length;
            case ':_':
                return coerce(node, "array", true).flat(Infinity);
            case '^*':
                return coerce(node, "int") > 0 && Math.sqrt(coerce(node, "int")) % 1 === 0;
            case ':n':
                return coerce(node, "string").split("\n");
            case ':s':
                return coerce(node, "string").split(" ");
            case ':{':
                return coerce(node, "array", true)[0];
            case ':}':
                let item = coerce(node, "array", true);
                return item[item.length - 1];
            case '.}':
                let drop_arr = coerce(node, "array", true);
                drop_arr.pop();
                return drop_arr;
            case '.{':
                let behead_arr = coerce(node, "array", true);
                behead_arr.shift();
                return behead_arr;
            case ':@':
                const repair = entry => entry instanceof BigNumber ? entry.toNumber() : entry;
                let arr = coerce(node, "array").map(repair);
                // Splice first element so reduce will work properly
                arr = [arr[0], ...arr];
                    
                return arr.reduce((acc, val) => typeof acc === "object" ? (acc.filter(entry => entry[0] === val).length ? (acc[acc.indexOf(acc.filter(entry => entry[0] === val)[0])].push(val), acc) : (acc.push([val]), acc)) : [[val]]);
            case '.@':
                let vec = coerce(node, "array", true).map(r => cast(r, "array"));

                return zip(...vec);
            case '.|':
                return coerce(node, "int").abs().toString();
            default:
                throw ArnError("Couldn't recognize suffix.", original, node.line, node.pos);
        }
    }

    var env = new Environment(false, original);
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
                env.create_func(node.value, node.args, node.body);
                break;
            case "call":
                let [arg_list, body] = env.get_func(node.value, node.line, node.pos);
                if (arg_list && arg_list.filter(r => r.type !== "variable").length > 0) throw new ArnError("Cannot pass non-variables as argument names to function.", original, node.line, node.pos);
                child_env = env.clone();

                if (arg_list) for (let i in arg_list) {
                    child_env.set(arg_list[i].value, constructType(evalNode(node.args[i], env)));
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
                ret_val = evalNode(env.get(node.value, node.line, node.pos), env, fix);
                break;
            case "javascript":
                node.body(env);
                break;
            default:
                throw new Error("Unrecognized node in AST:", JSON.stringify(node));
        }

        return ret_val;
    }
    
    if (opts.long.length) stdin = opts.long;

    function define_func(name, args, fn) {
        env.create_func(name, args, ast(tokenize(fn), original));
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
            }}) : [{type: "string", value: ""}]
        }
    });

    let item;
    if ((item = env.get("_").contents.contents).length === 1) env.set("_", item[0]);

    let std = [{type: "variable", value: "_"}];

    define_func("max", std, "(:<):{");
    define_func("min", std, "(:>):{");
    hardcode("out", std, (env) => printf(evalNode(env.get("_"), env, true)));
    hardcode("in", [], (env) => stdin);
    define_func("intr", std.concat([{type: "variable", value: "sep"}]), "|\\ (@| sep)");
    define_func("fact", std, "*\\ 1=>");
    define_func("mean", std, "(+\\) / #");
    define_func("mode", std, "(:< :@) :{:{");
    define_func("sdev", std, ":/mean(n{:*n-.mean}\\");
    
    return evalNode(tree, env);
}
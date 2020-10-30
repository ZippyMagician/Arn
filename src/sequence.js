function compare(original, partial) {
    return !Object.keys(partial).some((key) => partial[key] !== original[key]);
}

function copy(obj) {
    return JSON.parse(JSON.stringify(obj));
}

// A sequence can be infinite or finite
module.exports.Sequence = class Sequence {
    constructor (constants, block, length, env, evalNode) {
        this.constant = constants;
        this._built = this.constant;
        this.block = { ...block };
        this._backup = { ...block };

        this._env = env;
        this._evalNode = evalNode;
        this.len = length;

        this._index = 0;

        this.cur_offset = 0;
    }

    setEnv(env) {
        this._env = env;
        return this;
    }

    setEval(evalNode) {
        this._evalNode = evalNode;
        return this;
    }

    // TODO: Fix replacement
    _iterReplace(node) {
        let constructed = {};

        constructed.type = node.type;
        constructed.value = node.value;

        if (node.contents) {
            constructed.contents = {type: "prog", contents: []};
            constructed.contents.contents = node.contents.contents.map(entry => this._iterReplace({ ...entry }, this.cur_offset));
        }
        if (node.left) {
            constructed.left = this._iterReplace({ ...node.left }, this.cur_offset);
        }
        if (node.arg && typeof node.arg === "object") {
            constructed.arg = this._iterReplace({ ...node.arg }, this.cur_offset);
        } else if (node.arg) {
            constructed.arg = node.arg;
        }
        if (node.right) {
            constructed.right = this._iterReplace({ ...node.right }, this.cur_offset);
        }
        
        if (compare(node, { type: 'variable', value: '_' })) {
            // idk why I have to do this, TODO: Look for fix
            constructed = require('./formatter.js').constructType(this._built[this.cur_offset]);
            this.cur_offset -= 1;
        }
        
        return constructed;
    }

    _next() {
        if (this._index < this._built.length) {
            return this._built[this._index++];
        } else {
            let newBlock = { ...this.block };
            this.cur_offset = this._index - 1;
            newBlock = this._iterReplace(newBlock);

            this._built.push(this._evalNode(newBlock, this._env));
            this._index += 1;

            // Hacky. TODO: Fix
            this.block = { ...this._backup };

            return this._built[this._index - 1];
        }
    }

    _reset() {
        this._index = 0;
    }

    map(call) {
        let built = [];
        if (!this.len) throw new RangeError("Cannot map an infinite sequence");
        else {
            while (this._index < this.len) {
                built.push(call(copy(this._next())));
            }
        }

        this._reset();
        return built;
    }

    filter(call) {
        let built = copy(this._built);
        if (!this.len) throw new RangeError("Cannot map an infinite sequence");
        else {
            while (this._index < this.len) {
                let rem = call(copy(this._next()));
                if (!rem) delete built[this._index - 1];
            }

            built = built.filter(r => r);
        }

        this._reset();
        return built;
    }

    forEach(call) {
        if (!this.len) {
            while (true) {
                call(this._next());
            }
        } else {
            while (this._index < this.len) {
                call(this._next());
            }
        }

        this._reset();
    }

    take(count) {
        let constructed = [];
        if (!this.len) {
            while (this._index < count) {
                constructed.push(this._next());
            }
        } else {
            while (this._index < this.len && this._index < count) {
                constructed.push(this._next());
            }
        }
        this._reset();
        return constructed;
    }

    join(sep) {
        let built = [];
        this.forEach(entry => built.push(entry));
        return built.join(sep);
    }

    get(index) {
        while (index >= this._built.length) this._next();
        let ret = this._built[index];
        this._reset();
        return ret;
    }
}
class Environment {
    constructor(parent) {
        this.parent = parent;
        this.storage = [];
        this.func_storage = [];

        if (this.parent) {
            for (let entry of this.parent.storage) {
                this.storage.push(entry);
            }

            for (let entry of this.parent.func_storage) {
                this.func_storage.push(entry);
            }
        }
    }

    set(name, value) {
        if (this._exists(name)) {
            this.storage = this.storage.map(entry => entry.name === name ? {name, value} : entry);
        } else {
            this.storage.push({name, value});
        }
    }

    get(name) {
        if (this._exists(name)) {
            return this.storage.filter(r => r.name === name)[0].value;
        } else {
            throw new SyntaxError("Unrecognized variable: " + name);
        }
    }

    _exists(name) {
        return this.storage.filter(r => r.name === name).length > 0;
    }

    create_func(name, args, body) {
        this.func_storage.push({name, args, body});
    }

    get_func(name) {
        let filter = this.func_storage.filter(r => r.name === name);

        if (filter.length > 0) {
            return [filter[0].args, filter[0].body];
        } else {
            throw new SyntaxError("Unrecognized function: " + name);
        }
    }

    update(environment, exclude = "_") {
        for (let item of environment.storage) {
            if (this._exists(item.name) && item.name !== exclude) {
                this.set(item.name, item.value);
            }
        }
    }

    clone() {
        return new Environment(this);
    }
}

module.exports.Environment = Environment;
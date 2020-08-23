const { default: BigNumber } = require('bignumber.js');
const { Sequence } = require('./sequence.js');

// Prints in a formatted way
module.exports.printf = function printf(item, nested = false) {
    switch (typeof item) {
        case 'string':
        case 'number':
        case 'boolean':
            console.log(item.toString());
            break;
        case 'object':
            if (item instanceof BigNumber) console.log(item.toString());
            else if (nested) console.log(item.join(" "));
            else {
                item.forEach(entry => {
                    if (typeof entry === "object") {
                        if (entry instanceof BigNumber) {
                            console.log(entry.toString());
                        } else {
                            printf(entry, true);
                        }
                    } else {
                        console.log(entry.toString());
                    }
                });
            }
            break;
    }
}

module.exports.sprintf = function sprintf(item, nested = false) {
    let ret = "";
    switch (typeof item) {
        case 'string':
        case 'number':
            ret = item.toString();
            break;
        case 'object':
            if (item instanceof BigNumber) ret = item.toString();
            else if (nested) ret = item.join(" ");
            else {
                item.forEach(entry => {
                    if (typeof entry === "object") {
                        if (entry instanceof BigNumber) {
                            ret += entry.toString() + "\n";
                        } else {
                            ret += sprintf(entry, true) + "\n";
                        }
                    } else {
                        ret += entry.toString() + "\n";
                    }
                });
            }
            break;
    }

    return ret.trim();
}

// Casts between types
module.exports.cast = function cast(value, type) {
    switch (type) {
        case "int":
            return new BigNumber(typeof value === "object" ? value instanceof Sequence ? value.get(0) : value instanceof BigNumber ? value.toString() : value[0] : value);
        case "string":
            return typeof value === "string" ? value : typeof value === "number" ? value.toString() : value instanceof Sequence ? value.get(0) : value[0];
        case "array":
            return typeof value === "string" || typeof value === "number" ? value.toString().split(value.toString().indexOf(" ") > -1 ? " " : "") : value;
    }
}

// Gets the type of an item and returns the appropriate object
module.exports.constructType = function constructType(value) {
    switch (typeof value) {
        case 'string':
            return {type: "string", value: value};
        case 'number':
            return {type: "integer", value: +value};
        case 'object':
            if (value instanceof BigNumber) {
                return {type: "integer", value: value.toString()};
            }
            return {type: "array", contents: {type: "prog", contents: value.map(r => constructType(r))}};
        default:
            throw new Error("Could not construct type from", value);
    }
}

module.exports.stringify = val => {
    if (typeof val === "string") {
        return `"${val}"`;
    } else if (typeof val === "object") {
        if (val instanceof BigNumber) return `${val.toString()}`;
        else return `[${val.toString().replace(/,/g, " ")}]`;
    } else {
        return +val;
    }
}
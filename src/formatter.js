const { default: BigNumber } = require('bignumber.js');
const Sequence = require('./sequence.js');

// Prints in a formatted way
module.exports.printf = function printf(item, nested = false) {
    switch (typeof item) {
        case 'string':
        case 'number':
            console.log(item.toString());
            break;
        case 'object':
            if (item instanceof BigNumber) console.log(item.toString());
            else {
                item.forEach(entry => {
                    if (typeof entry === "object") {
                        if (nested) {
                            console.log(entry instanceof BigNumber ? entry.toString() : entry.join(" "));
                        } else {
                            if (entry instanceof BigNumber) {
                                console.log(entry.toString());
                            } else {
                                printf(entry, true);
                            }
                        }
                    } else {
                        console.log(entry.toString());
                    }
                });
            }
            break;
    }
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
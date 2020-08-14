const { default: BigNumber } = require('bignumber.js');

module.exports.printf = function printf(item, nested = false) {
    switch (typeof item) {
        case 'string':
        case 'number':
            console.log(item.toString());
            break;
        case 'object':
            if (item instanceof BigNumber) console.log(item.toString());
            else {
                item.foreach(entry => {
                    if (typeof entry === "object" && !nested) {
                        printf(entry, true);
                    } else if (typeof entry === "object" && nested) {
                        if (entry instanceof BigNumber) console.log(entry.toString());
                        else console.log(entry.join(" "));
                    } else {
                        console.log(entry.toString());
                    }
                });
            }
            break;
    }
}

module.exports.cast = function cast(value, type) {
    switch (type) {
        case "int":
            return typeof value === "object" ? +value[0] : +value;
        case "string":
            return typeof value === "string" ? value : typeof value === "number" ? value.toString() : value[0];
        case "array":
            return typeof value === "string" || typeof value === "number" ? value.toString().split(value.toString().indexOf(" ") > -1 ? " " : "") : value;
    }
}
const { constructArea } = require('./formatter.js');

module.exports.ArnError = class ArnError {
    constructor (msg, code, line, index) {
        return msg + "\n" + constructArea(code, line, index);
    }
}
const { constructArea } = require('./formatter.js');

class ArnError {
    constructor (msg, code, line, index) {
        this.msg = msg + "\n" + constructArea(code, line, index);
    }
}

module.exports.ArnError = function (msg, code, line, index) {
    let error = new ArnError(msg, code, line, index);
    return error.msg;
}
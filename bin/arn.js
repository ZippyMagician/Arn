#!/usr/bin/env node
var argv = require('minimist')(process.argv.slice(2));
const execute = require('../src');
const fs = require('fs');

class Options {
    constructor(argv) {
        this.long = [];
        this.stdin = [];

        this.long = argv._;
        this.stdin = argv.stdin;
        this.c = argv.c;
    }
}

let opts = new Options(argv);

switch (opts.long[0]) {
    case 'run':
        let file = opts.long[1];
        if (/^[A-Z]:/g.test(file)) {
            fs.readFile(file, 'utf8', (err, data) => {
                let str = String.raw`${data}`;
                execute(str, opts);
            });
        } else {
            fs.readFile(__dirname + '/' + file, 'utf8', (err, data) => {
                let str = String.raw`${data}`;
                execute(str, opts);
            });
        }
        break;
}
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
        this.u = argv.u;
    }
}

let opts = new Options(argv);

switch (opts.long[0]) {
    case 'run':
        if (opts.u) {
            execute(opts.u, opts);
            break;
        }
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
    case 'help':
        console.log("Here is a list of commands:\n * run -> Runs a file, or program directly if passed the -u flag\n * help -> Gets a list of all commands/flags\n\nHere are a list of flags:\n * -u -> Take manually inputted program instead of file\n * -c -> Compiles the code instead of running it");
        break;
}
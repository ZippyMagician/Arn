#!/usr/bin/env node
var argv = require('minimist')(process.argv.slice(2));
const { run: execute, parse } = require('../src');
const { printf } = require('../src/formatter.js');
const fs = require('fs');
const rl = require('readline-sync');
require('colors');

class Options {
    constructor(argv) {
        this.long = [];

        this.long = argv._;
        Object.assign(this, argv);

        if (this.stdin) {
            let file = this.stdin;
            if (/^[A-Z]:/g.test(file)) {
                fs.readFile(file, 'utf8', (err, data) => {
                    let str = String.raw`${data}`;
                    this.long.push(...str.split('\r\n'));
                });
            } else {
                fs.readFile(process.cwd() + '/' + file, 'utf8', (err, data) => {
                    let str = String.raw`${data}`;
                    this.long.push(...str.split('\r\n'));
                });
            }
        }
    }
}

let opts = new Options(argv);

switch (opts.long[0]) {
    case 'run':
        if (opts.u) {
            opts.long = opts.long.slice(1);
            execute(opts.u, opts);
            break;
        }
        let file = opts.long[1];
        opts.long = opts.long.slice(2);
        
        if (/^[A-Z]:/g.test(file)) {
            fs.readFile(file, 'utf8', (err, data) => {
                let str = String.raw`${data}`;
                execute(str, opts);
            });
        } else {
            fs.readFile(process.cwd() + '/' + file, 'utf8', (err, data) => {
                let str = String.raw`${data}`;
                execute(str, opts);
            });
        }
        break;
    case 'help':
        console.log("Here is a list of commands:\n * run -> Runs a file, or program directly if passed the -u flag\n * help -> Gets a list of all commands/flags\n\nHere are a list of flags:\n * -u -> Take manually inputted program instead of file\n * -c -> Compiles the code instead of running it\n * -d -> Will print some debug information (will be expanded in the future)\n * -p, --precision -> Sets the precision of Arn's BigNumbers. Default: 25 digits\n * --stdin -> Pass a file that contains STDIN. Useful if STDIN contains characters that may mess up when passed to the command");
        break;
    default:
        let inp = rl.question(">> ");
        while (inp !== "close") {
            try {
                printf(parse(inp, new Options({_:[]})));
            } catch (e) {
                console.log(e);
            }

            inp = rl.question(">> ");
        }
        break;
}
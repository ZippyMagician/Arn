#!/usr/bin/env node
var argv = require('minimist')(process.argv.slice(2));
const { run: execute, parse } = require('../src');
const { printf } = require('../src/formatter.js');
const fs = require('fs');
const rl = require('readline-sync');
require('colors');

const help_msg = `
Use the command arn to open the cli version of Arn

Here is a list of subcommands:
 * run -> Runs a file, or program directly if passed the -u flag
 * help -> Gets a list of all commands/flags

Here are a list of flags:
 * -u -> Take manually inputted program instead of file
 * -c -> Compiles the code instead of running it
 * -p, --precision -> Sets the precision of Arn's BigNumbers. Default: 25 digits
 * --stdin -> Pass a file that contains STDIN. Useful if STDIN contains characters that may mess up when passed to the command
 * -h -> Set STDIN to the range [1, 100]
 * -t -> Set STDIN to the range [1, 10]
 * -a -> Implicitly wrap program in array brackets
 * -e -> Evals STDIN as arn code
 * -r -> Changes STDIN from N to the range [1, N]
 * -m -> Wraps the program ... into {...}\\
 * -s -> Returns length of yielded value
 * -f -> First entry in yielded value
 * -l -> Last entry in yielded value
`;

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

    dummy() {
        let opts = new Options({_:[]});
        Object.assign(opts, this);
        opts.long = [];
        opts._ = [];

        return opts;
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
        console.log(help_msg.trim());
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
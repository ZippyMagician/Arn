// Pulls out all fixes, their rank, and definitions from the rust program and formats them for the wiki.
const fs = require('fs');

async function yank() {
    let consts = fs.readFileSync(__dirname + "/../src/utils/consts.rs").toString();
    let help = fs.readFileSync(__dirname + "/../src/parser.rs").toString();
    let defs = {};

    consts.replace(/['"](.{1,3})['"]: (.{1,2}); (.-.)/g, (_, name, prec, ranks) => defs[name.replace("\\\\", "\\")] = [prec, ranks.split("-")]);
    help.replace(/\/\/\s*(.+)\n\s*\"(.{1,3})\" =>/g, (_, help, name) => defs[name.replace("\\\\", "\\")].push(help));
    help.replace(/\/\/\s*(.+)\n\s*\"(.{1,3})\" \| \"(.{1,3})\" =>/g, (_, helps, n1, n2) => {
        helps = helps.split(",").map(n => n.trim());
        n1 = n1.replace("\\\\", "\\");
        n2 = n2.replace("\\\\", "\\");
        defs[n1].push(helps[0]);
        defs[n2].push(helps[1]);
    });

    let base = "|";
    let titles = ["Symbol", "Prec", "Rank", "About"];
    for (let i of titles) base += ` \`${i}\` |`;
    base += '\n'
    for (let _ in titles) base += '| :---: ';
    base += '|\n';
    
    for (n in defs) {
        base += `| \`${n.replace(/([\|])/g, "\\$1")}\` | \`${defs[n][0]}\` | \`${defs[n][1].map(c => " _ ".repeat(c)).join(n.replace(/([\|])/g, "\\$1")).trim()}\` | **${defs[n][2].replace(/([\<\>\|])/g, "\\$1")}** |\n`;
    }

    console.log(base);
}

yank();
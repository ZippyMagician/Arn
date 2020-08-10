// Generates code page table
const codePage = `!"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_\`abcdefghijklmnopqrstuvwxyz{|}~¡¢£¤¥¦§¨©ª«¬®¯°○■↑↓→←║═╔╗╚╝░▒►◄│─┌┐└┘├┤┴┬♦┼█▄▀▬±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿŒœŠšŸŽžƒƥʠˆ˜–—‘’‚“”„†‡•…‰‹›€™⁺⁻⁼`.split("");

function create_codepage() {
    let base = "| `_` |";
    for (let i of [...'0123456789ABCDEF']) base += ` \`_${i}\` |`;
    base += '\n';
    for (let i = 0; i < 17; i++) base += '| :---: ';

    let start = 0;
    base += '\n';

    for (let i of [...'0123456789ABCDEF']) {
        base += `| **\`${i}_\`** |`;
        for (let i = start; i < start + 16; i++) base += ` \`${codePage[i] === undefined ? "" : codePage[i]}\` |`;
        base += '\n';
        start += 16;
    }

    console.log(base);
}

create_codepage();
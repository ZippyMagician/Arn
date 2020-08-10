// Compresses the standard printable ascii characters into a modified version of CP1252
// Converts the bytes into base 95 (printable ascii), and then reads them off in base 252 (the amount of characters in my Code Page). 
// The inverse occurs to decompress
// I have found this to be shorter then creating a hash of all the 7-bit ascii characters and reading off 8-bits at a time
// Which was was I was doing before (it lead to an average compression rate of 11%, instead of this version's 17%)

const codePage = `!"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_\`abcdefghijklmnopqrstuvwxyz{|}~¡¢£¤¥¦§¨©ª«¬®¯°○■↑↓→←║═╔╗╚╝░▒►◄│─┌┐└┘├┤┴┬♦┼█▄▀▬±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿŒœŠšŸŽžƒƥʠˆ˜–—‘’‚“”„†‡•…‰‹›€™⁺⁻⁼`.split("");

module.exports.pack = (code) => {
  let bytes = [...code].map(r => r.charCodeAt(0) - 32);

  bytes = packBytes(bytes);
  return bytes.map(r => codePage[r]).join("");
}

function packBytes(bytes) {
  let result = [];
  let big = 0n;

  for (let i = bytes.length - 1; i >= 0; i--) {
    big = big * 95n + BigInt(bytes[i]);
  }

  while (big > 0) {
    result.push(big % 252n);
    big /= 252n;
  }

  return result;
}

module.exports.unpack = (packed) => {
  let bytes = [...packed].map(r => codePage.indexOf(r));

  bytes = unpackBytes(bytes);
  return bytes.map(r => String.fromCharCode((r + 32n).toString())).join("");
}

function unpackBytes(bytes) {
  let big = 0n;
  let result = [];

  for (let i = bytes.length - 1; i >= 0; i--) {
      big = big * 252n + BigInt(bytes[i]);
  }

  while (big > 0) {
      result.push(big % 95n);
      big /= 95n;
  }

  return result;
}

module.exports.isPacked = (code) => {
  return module.exports.unpack(module.exports.pack(code)) !== code;
}
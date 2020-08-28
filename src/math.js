module.exports.listPrimes = function primeSieve(n) {
    var a = Array(n = Math.ceil(n / 2)),
        t = (Math.sqrt(4 + 8 * n) - 2) / 4,
        u = 0,
        r = [];
    
    for (let i = 1; i <= t; i++) {
        u = (n - i) / (1 + 2 * i);
        for (let j = i; j <= u; j++) a[i + j + 2 * i * j] = true;
    }

    for (let i = 0; i <= n; i++) !a[i] && r.push((i * 2 + 1).toString());

    // Remove the last element if it's greater than the inputted number
    if (r[r.length - 1] > n * 2) r.pop();
    return r;
}
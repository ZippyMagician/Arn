// Newlines separate different test suites
function runArn(t, inp) {
    inp = inp.split("\n");
    if (inp.length === 1) {
        try {
            return sprintf(parse(t, {stdin: inp[0] || ""}));
        } catch (error) {
            return t + "\n" + error;
        }
    } else {
        let output = "";
        let count = 1;
        for (let suite of inp) {
            output += `* Case ${count++}:\n`;
            try {
                output += sprintf(parse(t, {stdin: suite || ""}));
            } catch (error) {
                output += t + "\n" + error;
            }
            output += "\n";
        }

        return output;
    }
}
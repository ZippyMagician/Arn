function runArn(t, inp) {
    try {
        return sprintf(parse(t, {stdin: inp || ""}));
    } catch (error) {
        return t + "\n" + error;
    }
}
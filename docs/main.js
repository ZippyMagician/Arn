// Newlines separate different test suites
function runArn(t, inp) {
    inp = inp.split("\n").map(r => r.replace(/\[N\]/g, "\n"));
    if (inp.length === 1) {
        try {
            return sprintf(parse(t, {stdin: inp[0].split("\n") || ""}));
        } catch (error) {
            return error;
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

// Example control
const examples = {
    'Hello World': [`'yt, bs!`, ``],
    'FizzBuzz': [`{("Fizz"^!%3)|(\`#&\`^!%5)||}\\~`, `100`],
    'Fibonacci': [`[1 1 {+} ->]`, `15`],
    'Evil Numbers': [`\${!(+\\;b)%2}~`, `400`],
    'Abundant Numbers': [`\${(+\\$v{!%v}1->)>}~`, '200']
}
  
for (const exampleName in examples) {
    if (examples.hasOwnProperty(exampleName)) {
        document.getElementById('demolist').innerHTML += `<li><a>${exampleName}</a></li>`
    }
}

$(document).on('click', '.dropdown-menu li a', function () {
    const val = $(this).html()
    $('#selectedbox').val(val)
  
    document.getElementById('code').value = examples[val][0];
    document.getElementById('ins').value = examples[val][1];
});
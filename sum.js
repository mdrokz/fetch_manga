const { readFileSync } = require('fs');


const sum = readFileSync('./count.txt')
    .toString()
    .split('\n')
    .map(x => parseInt(x))
    .filter(x => x > 0)
    .reduce((a, b) => a + b);

console.log(sum);
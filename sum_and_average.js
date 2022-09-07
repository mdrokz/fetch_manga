const { readFileSync } = require('fs');


const filteredCount = readFileSync('./count.txt')
    .toString()
    .split('\n')
    .map(x => parseInt(x))
    .filter(x => x > 0)


const sum = filteredCount.reduce((a, b) => a + b);

const average = sum / filteredCount.length;

console.log("SUM: " + sum, "AVERAGE: " + Math.round(average));
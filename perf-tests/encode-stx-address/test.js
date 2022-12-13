const sen = require('../..');
const fs = require('fs');
const path = require('path');

const startTime = Date.now();

const profile = sen.perfTestC32Encode();

const elapsed = Math.round((Date.now() - startTime) / 10) / 100;

const outputFile = path.join(__dirname, 'results', `profile-${Date.now()}-${elapsed}s.svg`);
fs.mkdirSync(path.dirname(outputFile), { recursive: true });
fs.writeFileSync(outputFile, profile);

console.log(`Took ${elapsed} seconds`);
console.log(`Output: ${outputFile}`);

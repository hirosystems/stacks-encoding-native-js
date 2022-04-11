let targetName = `${process.platform}-${process.arch}`;
if (process.platform === 'linux') {
    const libc = require('detect-libc').familySync() || 'glibc';
    targetName += `-${libc}`;
}
module.exports = require(require('path').resolve(__dirname, '../', 'native', `${targetName}.node`));

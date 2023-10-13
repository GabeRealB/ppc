const config = require('./webpack.config.js')();
const path = require('path');

config.entry = { main: './src/ts/demo/index.tsx' };
config.output = {
    filename: './output.js',
    path: path.resolve(__dirname),
};
config.performance = {
    hints: false
};
config.mode = 'development';
config.externals = undefined; // eslint-disable-line
config.devtool = 'inline-source-map';
module.exports = config;

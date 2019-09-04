const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const webpack = require('webpack');
const path = require('path');

module.exports = (env, argv) => {
  let mode = "development";
  if (argv && argv.mode) {
    mode = argv.mode;
  }

  console.log(`Building ${mode}...`);

  return {
    entry: path.resolve(__dirname, "js/index.js"),
    output: {
      path: path.resolve(__dirname, "build/dist"),
      filename: "ruffle.js",
    },
    mode: mode,
    plugins: []
  }
};

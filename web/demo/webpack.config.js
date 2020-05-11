/* eslint-env node */

const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const webpack = require('webpack');
const path = require('path');

module.exports = (env, argv) => {
  let mode = "production";
  if (argv && argv.mode) {
    mode = argv.mode;
  }

  console.log(`Building ${mode}...`);

  return {
    entry: path.resolve(__dirname, "www/index.js"),
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: "index.js",
    },
    mode: mode,
    plugins: [
      new CleanWebpackPlugin(),
      new CopyWebpackPlugin([{
        from: path.resolve(__dirname, "www/index.html"),
        to: "index.html"
      }]),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, ".."),
        outName: "ruffle",
        forceMode: mode,
      })
    ]
  }
};

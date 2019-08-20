const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const webpack = require('webpack');
const path = require('path');

module.exports = (env, argv) => {
  let mode = "development";
  if (argv && argv.mode) {
    mode = argv.mode;
  }

  console.log(`Building ${mode}...`);

  return {
    entry: path.resolve(__dirname, "www/bootstrap.js"),
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: "index.js",
    },
    mode: mode,
    plugins: [
      new CleanWebpackPlugin(),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, ".."),
        extraArgs: "--out-name=ruffle",
        forceMode: mode,
      })
    ]
  }
};

const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const CopyPlugin = require('copy-webpack-plugin');
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
    entry: path.resolve(__dirname, "js/index.js"),
    output: {
      path: path.resolve(__dirname, "build/dist"),
      filename: "ruffle.js",
      chunkFilename: "core.ruffle.js",
      jsonpFunction: "RufflePlayerExtensionLoader",
    },
    mode: mode,
    plugins: [
      new CleanWebpackPlugin(),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, ".."),
        outName: "ruffle",
        forceMode: mode,
      })
    ]
  }
};

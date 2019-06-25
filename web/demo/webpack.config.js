const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const webpack = require('webpack');
const path = require('path');

module.exports = {
  entry: path.resolve(__dirname, "www/bootstrap.js"),
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "development",
  plugins: [
    new CleanWebpackPlugin(),
    new CopyWebpackPlugin([{
      from: path.resolve(__dirname, "www/index.html"),
      to: "index.html"
    }]),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, ".."),
      extraArgs: "--out-name=ruffle",
    })
  ]
};

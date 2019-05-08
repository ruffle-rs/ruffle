const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const webpack = require('webpack');
const path = require('path');

module.exports = {
  entry: path.resolve(__dirname, "www/bootstrap.js"),
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin([{
      from: path.resolve(__dirname, "www/index.html"),
      to: "index.html"
    }]),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "."),
      extraArgs: "--out-name=ruffle",
    }),
    // Have this example work in Edge which doesn't ship `TextEncoder` or
    // `TextDecoder` at this time.
    new webpack.ProvidePlugin({
      TextDecoder: ['text-encoding', 'TextDecoder'],
      TextEncoder: ['text-encoding', 'TextEncoder']
    })
  ],
};

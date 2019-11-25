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
    externals: /^(ruffle_web)$/i,

    entry: path.resolve(__dirname, "js/index.js"),
    output: {
      path: path.resolve(__dirname, "build/dist"),
      filename: "ruffle.js",
    },
    mode: mode,
    plugins: [
      new webpack.IgnorePlugin({
        resourceRegExp: /..\/pkg\/ruffle/
      }),
      new CleanWebpackPlugin({
        protectWebpackAssets: false,
        cleanAfterEveryBuildPatterns: ["*.module.wasm"],
      }),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, ".."),
        extraArgs: "--target=no-modules",
        forceMode: mode,
      }),
      new CopyPlugin([
        { from: "../pkg/ruffle_web.js", to: "ruffle_web.js" },
        { from: "../pkg/ruffle_web_bg.wasm", to: "ruffle_web_bg.wasm" },
      ])
    ]
  }
};

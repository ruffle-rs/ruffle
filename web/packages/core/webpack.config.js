/* eslint-env node */

const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const path = require("path");

module.exports = (env, argv) => {
    let extraArgs = "";

    let mode = "production";
    if (argv && argv.mode) {
        mode = argv.mode;
    }

    if (env && env.features) {
        extraArgs = `-- --features ${env.features}`;
    }

    console.log(`Building ${mode}...`);

    return {
        entry: path.resolve(__dirname, "index.js"),
        output: {
            path: path.resolve(__dirname, "dist"),
            filename: "ruffle.js",
            chunkFilename: "core.ruffle.[contenthash].js",
            jsonpFunction: "RufflePlayerLoader",
        },
        mode: mode,
        plugins: [
            new CleanWebpackPlugin(),
            new WasmPackPlugin({
                crateDirectory: path.resolve(__dirname, "../.."),
                outName: "ruffle",
                forceMode: mode,
                extraArgs,
            }),
        ],
    };
};

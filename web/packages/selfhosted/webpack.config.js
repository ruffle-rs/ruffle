/* eslint-env node */

const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const CopyPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = (env, argv) => {
    let mode = "production";
    if (argv && argv.mode) {
        mode = argv.mode;
    }

    console.log(`Building ${mode}...`);

    return {
        entry: path.resolve(__dirname, "js/ruffle.js"),
        output: {
            path: path.resolve(__dirname, "dist"),
            filename: "ruffle.js",
            chunkFilename: "core.ruffle.[contenthash].js",
            jsonpFunction: "RufflePlayerLoader",
        },
        mode: mode,
        plugins: [
            new CleanWebpackPlugin(),
            new CopyPlugin({
                patterns: [{ from: "LICENSE*" }, { from: "README.md" }],
            }),
        ],
    };
};

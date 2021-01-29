/* eslint-env node */

const path = require("path");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");

module.exports = (env, argv) => {
    let mode = "production";
    if (argv && argv.mode) {
        mode = argv.mode;
    }

    console.log(`Building ${mode}...`);

    return {
        mode,
        entry: path.resolve(__dirname, "js/ruffle.js"),
        output: {
            path: path.resolve(__dirname, "dist"),
            filename: "ruffle.js",
            publicPath: "",
            chunkFilename: "core.ruffle.[contenthash].js",
        },
        module: {
            rules: [
                {
                    test: /\.wasm$/i,
                    use: ["file-loader"],
                },
            ],
        },
        devtool: "source-map",
        plugins: [
            new CleanWebpackPlugin(),
            new CopyWebpackPlugin({
                patterns: [{ from: "LICENSE*" }, { from: "README.md" }],
            }),
        ],
    };
};

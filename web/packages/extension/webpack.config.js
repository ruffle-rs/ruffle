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
        entry: {
            ruffle: path.resolve(__dirname, "js/index.js"),
            popup: path.resolve(__dirname, "js/popup.js"),
            settings: path.resolve(__dirname, "js/settings.js"),
            lv0: path.resolve(__dirname, "js/lv0.js"),
        },
        output: {
            path: path.resolve(__dirname, "build/dist"),
            filename: "[name].js",
            chunkFilename: "core.ruffle.js",
        },
        mode: mode,
        experiments: {
            syncWebAssembly: true,
        },
        plugins: [
            new CleanWebpackPlugin(),
            new CopyPlugin({
                patterns: [{ from: "LICENSE*" }, { from: "README.md" }],
            }),
        ],
    };
};

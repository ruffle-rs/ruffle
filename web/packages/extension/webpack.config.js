/* eslint-env node */

const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const path = require("path");

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
        plugins: [new CleanWebpackPlugin()],
    };
};

/* eslint-env node */

const path = require("path");
const CopyWebpackPlugin = require("copy-webpack-plugin");

module.exports = (env, argv) => {
    let mode = "production";
    if (argv && argv.mode) {
        mode = argv.mode;
    }

    console.log(`Building ${mode}...`);

    return {
        mode,
        entry: {
            popup: path.resolve(__dirname, "src/popup.js"),
            options: path.resolve(__dirname, "src/options.js"),
            content: path.resolve(__dirname, "src/content.js"),
            ruffle: path.resolve(__dirname, "src/ruffle.js"),
        },
        output: {
            path: path.resolve(__dirname, "assets/dist/"),
            publicPath: "",
            clean: true,
        },
        module: {
            rules: [
                {
                    resource: path.resolve(__dirname, "src/pluginPolyfill.js"),
                    type: "asset/source",
                },
                {
                    test: /\.wasm$/i,
                    use: ["file-loader"],
                },
            ],
        },
        plugins: [
            new CopyWebpackPlugin({
                patterns: [{ from: "LICENSE*" }, { from: "README.md" }],
            }),
        ],
    };
};

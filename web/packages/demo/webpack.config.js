/* eslint-env node */

const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");

module.exports = (_env, _argv) => {
    const mode = process.env.NODE_ENV || "production";
    console.log(`Building ${mode}...`);

    return {
        mode,
        entry: "./www/index.js",
        output: {
            path: path.resolve(__dirname, "dist"),
            filename: "index.js",
            publicPath: "",
            clean: true,
        },
        module: {
            rules: [
                {
                    test: /\.css$/i,
                    use: ["style-loader", "css-loader"],
                },
            ],
        },
        performance: {
            assetFilter: (assetFilename) =>
                !/\.(map|wasm)$/i.test(assetFilename),
        },
        devtool: "source-map",
        plugins: [
            new CopyPlugin({
                patterns: [
                    { from: path.resolve(__dirname, "www/index.html") },
                    { from: "LICENSE*" },
                    { from: "README.md" },
                ],
            }),
        ],
    };
};

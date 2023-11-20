/* eslint-env node */

const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");

module.exports = (_env, _argv) => {
    const mode = process.env.NODE_ENV || "production";
    console.log(`Building ${mode}...`);

    return {
        mode,
        entry: "./src/index.ts",
        output: {
            path: path.resolve(__dirname, "dist"),
            filename: "index.js",
            publicPath: "",
            clean: true,
        },
        resolve: {
            extensions: [".ts", "..."],
        },
        module: {
            rules: [
                {
                    test: /\.ts$/i,
                    use: "ts-loader",
                },
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
        devServer: {
            client: {
                overlay: false,
            },
        },
        devtool: "source-map",
        plugins: [
            new CopyPlugin({
                patterns: [
                    { from: path.resolve(__dirname, "www/index.html") },
                    { from: path.resolve(__dirname, "www/logo-anim.swf") },
                    { from: path.resolve(__dirname, "www/icon32.png") },
                    { from: path.resolve(__dirname, "www/icon48.png") },
                    { from: path.resolve(__dirname, "www/icon180.png") },
                    { from: path.resolve(__dirname, "www/logo.svg") },
                    { from: "swfs.json", noErrorOnMissing: true },
                    { from: "LICENSE*" },
                    { from: "README.md" },
                ],
            }),
        ],
    };
};

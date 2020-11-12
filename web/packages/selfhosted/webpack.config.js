/* eslint-env node */

const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const CopyPlugin = require("copy-webpack-plugin");
const webpack = require("webpack");
const path = require("path");

module.exports = (env, argv) => {
    let mode = "production";
    if (argv && argv.mode) {
        mode = argv.mode;
    }

    const commitHash = require("child_process")
        .execSync("git rev-parse --short HEAD")
        .toString();

    const commitDate = require("child_process")
        .execSync("git log -1 --date=short --pretty=format:%cd")
        .toString();

    const channel = process.env.CFG_RELEASE_CHANNEL || "nightly".toLowerCase();

    console.log(`Building ${mode}...`);

    return {
        entry: path.resolve(__dirname, "js/ruffle.js"),
        output: {
            publicPath: "",
            path: path.resolve(__dirname, "dist"),
            filename: "ruffle.js",
            chunkFilename: "core.ruffle.[contenthash].js",
        },
        mode: mode,
        experiments: {
            syncWebAssembly: true,
        },
        devtool: "source-map",
        resolve: {
            extensions: [".ts", ".tsx", ".js", ".wasm"],
        },
        plugins: [
            new CleanWebpackPlugin(),
            new webpack.DefinePlugin({
                __COMMIT_HASH__: JSON.stringify(commitHash),
                __COMMIT_DATE__: JSON.stringify(commitDate),
                __CHANNEL__: JSON.stringify(channel),
            }),
            new CopyPlugin({
                patterns: [{ from: "LICENSE*" }, { from: "README.md" }],
            }),
        ],
        module: {
            rules: [
                {
                    test: /\.tsx?$/,
                    loader: "ts-loader",
                },
            ],
        },
    };
};

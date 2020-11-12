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
            new WasmPackPlugin({
                crateDirectory: path.resolve(__dirname, "../.."),
                outName: "ruffle",
                forceMode: mode,
                extraArgs,
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

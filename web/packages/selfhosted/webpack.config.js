/* eslint-env node */

const path = require("path");
const json5 = require("json5");
const CopyPlugin = require("copy-webpack-plugin");

function transformPackage(content) {
    const package = json5.parse(content);

    const packageVersion = process.env.npm_package_version;

    const versionChannel = process.env.CFG_RELEASE_CHANNEL || "nightly";

    const buildDate = new Date()
        .toISOString()
        .substring(0, 10)
        .replace(/-/g, ".");

    // The npm registry requires the version to monotonically increase,
    // so append the build date onto the end of the package version.
    package.version =
        versionChannel !== "stable"
            ? `${packageVersion}-${versionChannel}.${buildDate}`
            : packageVersion;

    return JSON.stringify(package);
}

module.exports = (_env, _argv) => {
    const mode = process.env.NODE_ENV || "production";
    console.log(`Building ${mode}...`);

    return {
        mode,
        entry: "./js/ruffle.js",
        output: {
            path: path.resolve(__dirname, "dist"),
            filename: "ruffle.js",
            publicPath: "",
            chunkFilename: "core.ruffle.[contenthash].js",
            clean: true,
        },
        performance: {
            assetFilter: (assetFilename) =>
                !/\.(map|wasm)$/i.test(assetFilename),
        },
        devtool: "source-map",
        plugins: [
            new CopyPlugin({
                patterns: [
                    {
                        from: "npm-package.json5",
                        to: "package.json",
                        transform: transformPackage,
                    },
                    { from: "LICENSE*" },
                    { from: "README.md" },
                ],
            }),
        ],
    };
};

/* eslint-env node */

const path = require("path");
const json5 = require("json5");
const CopyPlugin = require("copy-webpack-plugin");

function transformManifest(content, env) {
    const manifest = json5.parse(content);

    const packageVersion = process.env.npm_package_version;

    const versionChannel = process.env.CFG_RELEASE_CHANNEL || "nightly";

    const buildDate = new Date().toISOString().substring(0, 10);

    // The extension marketplaces require the version to monotonically increase,
    // so append the build date onto the end of the manifest version.
    manifest.version = process.env.BUILD_ID
        ? `${packageVersion}.${process.env.BUILD_ID}`
        : packageVersion;

    if (env.firefox) {
        manifest.browser_specific_settings = {
            gecko: {
                id: process.env.FIREFOX_EXTENSION_ID || "ruffle@ruffle.rs",
                strict_min_version: "91.1.0",
            },
        };
    } else {
        manifest.version_name =
            versionChannel === "nightly"
                ? `${packageVersion} nightly ${buildDate}`
                : packageVersion;
    }

    return JSON.stringify(manifest);
}

module.exports = (env, _argv) => {
    const mode = process.env.NODE_ENV || "production";
    console.log(`Building ${mode}...`);

    return {
        mode,
        entry: {
            popup: "./src/popup.ts",
            options: "./src/options.ts",
            content: "./src/content.ts",
            ruffle: "./src/ruffle.ts",
            background: "./src/background.ts",
            player: "./src/player.ts",
        },
        output: {
            path: path.resolve(__dirname, "assets/dist/"),
            publicPath: "",
            clean: true,
        },
        module: {
            rules: [
                {
                    test: /\.ts$/i,
                    use: "ts-loader",
                },
            ],
        },
        resolve: {
            extensions: [".ts", "..."],
        },
        plugins: [
            new CopyPlugin({
                patterns: [
                    {
                        from: "manifest.json5",
                        to: "../manifest.json",
                        transform: (content) => transformManifest(content, env),
                    },
                    { from: "LICENSE*" },
                    { from: "README.md" },
                ],
            }),
        ],
    };
};

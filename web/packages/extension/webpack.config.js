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
    const version = process.env.BUILD_ID
        ? `${packageVersion}.${process.env.BUILD_ID}`
        : packageVersion;

    const version_name =
        versionChannel === "nightly"
            ? `${packageVersion} nightly ${buildDate}`
            : packageVersion;

    Object.assign(manifest, { version, version_name });
    if (env.firefox) {
        const id = process.env.FIREFOX_EXTENSION_ID || "ruffle@ruffle.rs";
        Object.assign(manifest, {
            browser_specific_settings: {
                gecko: { id },
            },
        });
    }

    return JSON.stringify(manifest);
}

module.exports = (env, argv) => {
    let mode = "production";
    if (argv && argv.mode) {
        mode = argv.mode;
    }

    console.log(`Building ${mode}...`);

    return {
        mode,
        entry: {
            popup: "./src/popup.js",
            options: "./src/options.js",
            content: "./src/content.js",
            ruffle: "./src/ruffle.js",
            background: "./src/background.js",
            player: "./src/player.js",
        },
        output: {
            path: path.resolve(__dirname, "assets/dist/"),
            publicPath: "",
            clean: true,
        },
        module: {
            rules: [
                {
                    test: /\.wasm$/i,
                    type: "asset/resource",
                },
            ],
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

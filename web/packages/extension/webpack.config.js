/* eslint-env node */

const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");

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
            background: path.resolve(__dirname, "src/background.js"),
            player: path.resolve(__dirname, "src/player.js"),
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
                    use: ["file-loader"],
                },
            ],
        },
        plugins: [
            new CopyPlugin({
                patterns: [
                    {
                        from: "manifest.json",
                        to: "..",
                        transform(content) {
                            const manifest = JSON.parse(content.toString());

                            const packageVersion =
                                process.env.npm_package_version;
                            const versionChannel =
                                process.env.CFG_RELEASE_CHANNEL || "nightly";
                            const buildDate = new Date()
                                .toISOString()
                                .substring(0, 10);
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
                                const id =
                                    process.env.FIREFOX_EXTENSION_ID ||
                                    "ruffle@ruffle.rs";

                                Object.assign(manifest, {
                                    browser_specific_settings: {
                                        gecko: { id },
                                    },
                                });
                            }
                            return JSON.stringify(manifest);
                        },
                    },
                    { from: "LICENSE*" },
                    { from: "README.md" },
                ],
            }),
        ],
    };
};

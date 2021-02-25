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
                patterns: [
                    {
                        from: "manifest.json",
                        to: "..",
                        transform(content) {
                            const manifest = JSON.parse(content.toString());
                            const { version } = require("./package");
                            Object.assign(manifest, { version });
                            if (env.firefox) {
                                const id =
                                    process.env.FIREFOX_EXTENSION_ID ||
                                    "ruffle-player-extension@ruffle.rs";
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

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
            lv0: path.resolve(__dirname, "src/lv0.js"),
            ruffle: path.resolve(__dirname, "src/index.js"),
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
                            const { version } = require("./package.json");
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

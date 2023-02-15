/* eslint-env node */

const path = require("path");
const json5 = require("json5");
const CopyPlugin = require("copy-webpack-plugin");
const fs = require("fs");

function transformManifest(content, env) {
    const manifest = json5.parse(content);

    let packageVersion = process.env.npm_package_version;
    let versionChannel = process.env.CFG_RELEASE_CHANNEL || "nightly";
    let buildDate = new Date().toISOString().substring(0, 10);
    let build_id = process.env.BUILD_ID;
    let firefox_extension_id =
        process.env.FIREFOX_EXTENSION_ID || "ruffle@ruffle.rs";

    if (process.env.ENABLE_VERSION_SEAL === "true") {
        if (fs.existsSync("../../version_seal.json")) {
            const version_seal = JSON.parse(
                fs.readFileSync("../../version_seal.json")
            );

            packageVersion = version_seal.version_number;
            versionChannel = version_seal.version_channel;
            buildDate = version_seal.build_date.substring(0, 10);
            build_id = version_seal.build_id;
            firefox_extension_id = version_seal.firefox_extension_id;
        } else {
            throw new Error(
                "Version seal requested but not found. Please run web/packages/core/tools/set_version.js with ENABLE_VERSION_SEAL to generate it."
            );
        }
    }

    // At this point all code below needs to be deterministic. If you want other
    // information to be included here you must store it in the version seal
    // when it gets generated in web/packages/core/tools/set_version.js and then
    // load it in the code above.

    // The extension marketplaces require the version to monotonically increase,
    // so append the build number onto the end of the manifest version.
    manifest.version = build_id
        ? `${packageVersion}.${build_id}`
        : packageVersion;

    if (env.firefox) {
        manifest.browser_specific_settings = {
            gecko: {
                id: firefox_extension_id,
            },
        };
    } else {
        manifest.version_name =
            versionChannel === "nightly"
                ? `${packageVersion} nightly ${buildDate}`
                : packageVersion;

        // Add `wasm-eval` to the `script-src` directive in the Content Security Policy.
        // This setting is required by Chrome to allow Wasm in the extension.
        // Eventually this may change to `wasm-unsafe-eval`, and we may need this for all browsers.
        manifest.content_security_policy =
            manifest.content_security_policy.replace(
                /(script-src\s+[^;]*)(;|$)/i,
                "$1 'wasm-eval'$2"
            );

        // Chrome runs the extension in a single shared process by default,
        // which prevents extension pages from loading in Incognito tabs
        manifest.incognito = "split";
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
            pluginPolyfill: "./src/plugin-polyfill.ts",
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
        performance: {
            assetFilter: (assetFilename) =>
                !/\.(map|wasm)$/i.test(assetFilename),
        },
        optimization: {
            minimize: false,
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

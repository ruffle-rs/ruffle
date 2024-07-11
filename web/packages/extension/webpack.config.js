import fs from "fs";
import url from "url";
import json5 from "json5";
import CopyPlugin from "copy-webpack-plugin";

/**
 * @param {Buffer} content
 * @param {Record<string, any>} env
 * @returns {string}
 */
function transformManifest(content, env) {
    const manifest = json5.parse(content.toString());

    let packageVersion = process.env["npm_package_version"];
    let versionChannel = process.env["CFG_RELEASE_CHANNEL"] || "nightly";
    let buildDate = new Date().toISOString().substring(0, 10);
    let buildId = process.env["BUILD_ID"];
    let firefoxExtensionId =
        process.env["FIREFOX_EXTENSION_ID"] || "ruffle@ruffle.rs";

    if (process.env["ENABLE_VERSION_SEAL"] === "true") {
        if (fs.existsSync("../../version_seal.json")) {
            const versionSeal = JSON.parse(
                fs.readFileSync("../../version_seal.json", "utf8"),
            );

            packageVersion = versionSeal.version_number;
            versionChannel = versionSeal.version_channel;
            buildDate = versionSeal.build_date.substring(0, 10);
            buildId = versionSeal.build_id;
            firefoxExtensionId = versionSeal.firefox_extension_id;
        } else {
            throw new Error(
                "Version seal requested but not found. To generate it, please run web/packages/core/tools/set_version.js using node in the web directory, with the ENABLE_VERSION_SEAL environment variable set to true.",
            );
        }
    }

    // At this point all code below needs to be deterministic. If you want other
    // information to be included here you must store it in the version seal
    // when it gets generated in web/packages/core/tools/set_version.js and then
    // load it in the code above.

    // The extension marketplaces require the version to monotonically increase,
    // so append the build number onto the end of the manifest version.
    manifest.version = buildId
        ? `${packageVersion}.${buildId}`
        : packageVersion;

    if (env["firefox"]) {
        manifest.browser_specific_settings = {
            gecko: {
                id: firefoxExtensionId,
            },
        };
        manifest.background = {
            scripts: ["dist/background.js"],
        };
    } else {
        manifest.version_name =
            versionChannel === "nightly"
                ? `${packageVersion} nightly ${buildDate}`
                : packageVersion;

        manifest.background = {
            service_worker: "dist/background.js",
        };

        // Chrome runs the extension in a single shared process by default,
        // which prevents extension pages from loading in Incognito tabs
        manifest.incognito = "split";
    }

    return JSON.stringify(manifest);
}

/**
 * @type {import("webpack-cli").CallableOption}
 */
export default function (/** @type {Record<string, any>} */ env, _argv) {
    const mode =
        /** @type {import("webpack").Configuration["mode"]} */ (
            process.env["NODE_ENV"]
        ) || "production";
    console.log(`Building ${mode}...`);

    return {
        mode,
        entry: {
            popup: "./src/popup.ts",
            options: "./src/options.ts",
            onboard: "./src/onboard.ts",
            content: "./src/content.ts",
            ruffle: "./src/ruffle.ts",
            background: "./src/background.ts",
            player: "./src/player.ts",
            pluginPolyfill: "./src/plugin-polyfill.ts",
            siteContentScript4399: "./src/4399-content-script.ts",
        },
        output: {
            path: url.fileURLToPath(new URL("assets/dist/", import.meta.url)),
            publicPath: "auto",
            clean: true,
            assetModuleFilename: "assets/[name][ext][query]",
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
            /**
             * @param {string} assetFilename
             * @returns {boolean}
             */
            assetFilter: (assetFilename) =>
                !/\.(map|wasm)$/i.test(assetFilename),
        },
        optimization: {
            minimize: false,
        },
        devtool: mode === "development" ? "source-map" : false,
        plugins: [
            new CopyPlugin({
                patterns: [
                    {
                        from: "manifest.json5",
                        to: "../manifest.json",
                        transform: (content) =>
                            transformManifest(
                                content,
                                /** @type {Record<string, any>} */ (env),
                            ),
                    },
                    { from: "LICENSE*" },
                    { from: "README.md" },
                    { from: "4399_rules.json" },
                ],
            }),
        ],
    };
}

import url from "url";
import json5 from "json5";
import CopyPlugin from "copy-webpack-plugin";
import TerserPlugin from "terser-webpack-plugin";
import fs from "fs";

function transformPackage(content) {
    const pkg = json5.parse(content);

    if (pkg["webpack-copy-properties"]) {
        const copyProperties = pkg["webpack-copy-properties"];
        delete pkg["webpack-copy-properties"];

        const otherPkg = json5.parse(
            fs.readFileSync(copyProperties.from, { encoding: "utf8" }),
        );
        for (const property of copyProperties.properties) {
            pkg[property] = otherPkg[property];
        }
    }

    return JSON.stringify(pkg);
}

export default function (_env, _argv) {
    const mode = process.env.NODE_ENV || "production";
    console.log(`Building ${mode}...`);

    return {
        mode,
        entry: "./js/ruffle.js",
        output: {
            path: url.fileURLToPath(new URL("dist", import.meta.url)),
            filename: "ruffle.js",
            publicPath: "",
            chunkFilename: "core.ruffle.[contenthash].js",
            clean: true,
        },
        performance: {
            assetFilter: (assetFilename) =>
                !/\.(map|wasm)$/i.test(assetFilename),
        },
        optimization: {
            minimizer: [
                new TerserPlugin({
                    terserOptions: {
                        output: {
                            ascii_only: true,
                        },
                    },
                }),
            ],
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
}

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, URL } from "node:url";
import json5 from "json5";
import * as esbuild from "esbuild";
import { wasmAssetsPlugin } from "../core/tools/esbuild_wasm_assets.ts";

interface VersionSeal {
    version_number?: string;
    version_channel?: string;
    version4?: string;
    firefox_extension_id?: string;
}

interface Manifest {
    version: string | undefined;
    version_name: string | undefined;
    browser_specific_settings?: {
        gecko?: {
            id?: string;
            data_collection_permissions?: {
                required: string[];
            };
            strict_min_version: string;
        };
        [key: string]: unknown;
    };
    background?: {
        scripts?: string[];
        service_worker?: string;
        [key: string]: unknown;
    };
    incognito?: string;
    [key: string]: unknown;
}

const __dirname: string = fileURLToPath(new URL(".", import.meta.url));
const assetsDir = path.join(__dirname, "assets");
const distDir = path.join(assetsDir, "dist");
const coreDir = path.resolve(__dirname, "../core");

const args: string[] = process.argv.slice(2);
const envIndex = args.indexOf("--env");
const isFirefox = envIndex !== -1 && args[envIndex + 1] === "firefox";

const mode: string = process.env["NODE_ENV"] || "production";
const isDevelopment: boolean = mode === "development";

// Clean the output directory
fs.rmSync(distDir, { recursive: true, force: true });
fs.mkdirSync(distDir, { recursive: true });

// Transform and write manifest.json to assets/
function transformManifest(): void {
    const rawContent: string = fs.readFileSync(
        path.join(__dirname, "manifest.json5"),
        "utf8",
    );
    const manifest = json5.parse(rawContent) as Manifest;

    let packageVersion: string | undefined = process.env["npm_package_version"];
    let versionChannel: string = process.env["CFG_RELEASE_CHANNEL"] || "local";
    let version4: string | undefined = process.env["VERSION4"];
    let firefoxExtensionId: string =
        process.env["FIREFOX_EXTENSION_ID"] || "ruffle@ruffle.rs";

    if (process.env["ENABLE_VERSION_SEAL"] === "true") {
        const sealPath = path.resolve(__dirname, "../../version_seal.json");
        if (fs.existsSync(sealPath)) {
            const versionSeal = JSON.parse(
                fs.readFileSync(sealPath, "utf8"),
            ) as VersionSeal;

            packageVersion = versionSeal.version_number;
            versionChannel = versionSeal.version_channel ?? versionChannel;
            version4 = versionSeal.version4;
            firefoxExtensionId =
                versionSeal.firefox_extension_id ?? firefoxExtensionId;
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

    // The extension marketplaces require the version to monotonically increase
    // and to be in the format of A.B.C.D.
    manifest.version = version4 || packageVersion;

    if (isFirefox) {
        manifest.browser_specific_settings = {
            gecko: {
                id: firefoxExtensionId,
                data_collection_permissions: {
                    required: ["none"],
                },
                strict_min_version: "128.0",
            },
        };
        manifest.background = {
            scripts: ["dist/background.js"],
        };
    } else {
        if (
            versionChannel === "stable" ||
            packageVersion?.includes(versionChannel)
        ) {
            manifest.version_name = packageVersion;
        } else {
            manifest.version_name = `${versionChannel} ${packageVersion}`;
        }

        manifest.background = {
            service_worker: "dist/background.js",
        };

        // Chrome runs the extension in a single shared process by default,
        // which prevents extension pages from loading in Incognito tabs
        manifest.incognito = "split";
    }

    fs.writeFileSync(
        path.join(assetsDir, "manifest.json"),
        JSON.stringify(manifest, null, 2),
    );
}

transformManifest();

// Copy static files to assets/
const rootFiles: string[] = fs.readdirSync(__dirname);
for (const file of rootFiles) {
    if (
        file.startsWith("LICENSE") ||
        file === "README.md" ||
        file === "4399_rules.json"
    ) {
        fs.copyFileSync(path.join(__dirname, file), path.join(distDir, file));
    }
}

// Bundle entry points with ESBuild into assets/dist/
await esbuild.build({
    entryPoints: {
        popup: "./src/popup.ts",
        options: "./src/options.ts",
        onboard: "./src/onboard.ts",
        content: "./src/content.ts",
        ruffle: "./src/ruffle.ts",
        background: "./src/background.ts",
        player: "./src/player.ts",
        pluginPolyfill: "./src/plugin-polyfill.ts",
        pluginPolyfillIgnoreOptout: "./src/plugin-polyfill-ignore-optout.ts",
        siteContentScript4399: "./src/4399-content-script.ts",
    },
    bundle: true,
    outdir: distDir,
    format: "iife",
    assetNames: "[name]",
    inject: [path.join(coreDir, "dist", "pristine-globals.js")],
    minify: false,
    sourcemap: isDevelopment ? "inline" : false,
    target: "es2021",
    plugins: [wasmAssetsPlugin()],
});

console.log(`ESBuild for ${mode} complete!`);

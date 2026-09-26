import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, URL } from "node:url";
import json5 from "json5";
import * as esbuild from "esbuild";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const assetsDir = path.join(__dirname, "assets");
const distDir = path.join(assetsDir, "dist");
const coreDistDir = path.resolve(__dirname, "../core/dist");

// Parse CLI flags (--env firefox / --env generic or --firefox)
const args = process.argv.slice(2);
const isFirefox =
    args.includes("--firefox") ||
    args.includes("firefox") ||
    (args.indexOf("--env") !== -1 &&
        args[args.indexOf("--env") + 1] === "firefox");

const env = { firefox: isFirefox, generic: !isFirefox };
const mode = process.env["NODE_ENV"] || "production";
const isDevelopment = mode === "development";

// Ensure assets/dist directory exists
// 2. Clean output directories
if (fs.existsSync(distDir)) {
    fs.rmSync(distDir, { recursive: true, force: true });
}
fs.mkdirSync(distDir, { recursive: true });
// 3. Copy ONLY .wasm binaries from ruffle-core
if (fs.existsSync(coreDistDir)) {
    const coreFiles = fs.readdirSync(coreDistDir);
    for (const file of coreFiles) {
        if (file.endsWith(".wasm")) {
            const srcPath = path.join(coreDistDir, file);
            fs.copyFileSync(srcPath, path.join(distDir, file));
        }
    }
}

// Transform and write manifest.json to assets/
function transformManifest() {
    const rawContent = fs.readFileSync(
        path.join(__dirname, "manifest.json5"),
        "utf8",
    );
    const manifest = json5.parse(rawContent);

    let packageVersion = process.env["npm_package_version"];
    let versionChannel = process.env["CFG_RELEASE_CHANNEL"] || "local";
    let version4 = process.env["VERSION4"];
    let firefoxExtensionId =
        process.env["FIREFOX_EXTENSION_ID"] || "ruffle@ruffle.rs";

    if (process.env["ENABLE_VERSION_SEAL"] === "true") {
        const sealPath = path.resolve(__dirname, "../../version_seal.json");
        if (fs.existsSync(sealPath)) {
            const versionSeal = JSON.parse(fs.readFileSync(sealPath, "utf8"));

            packageVersion = versionSeal.version_number;
            versionChannel = versionSeal.version_channel;
            version4 = versionSeal.version4;
            firefoxExtensionId = versionSeal.firefox_extension_id;
        } else {
            throw new Error(
                "Version seal requested but not found. To generate it, please run web/packages/core/tools/set_version.js using node in the web directory, with the ENABLE_VERSION_SEAL environment variable set to true.",
            );
        }
    }

    manifest.version = version4 ? version4 : packageVersion;

    if (env.firefox) {
        manifest.browser_specific_settings = {
            gecko: {
                id: firefoxExtensionId,
                data_collection_permissions: {
                    required: ["none"],
                },
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

        manifest.incognito = "split";
    }

    fs.writeFileSync(
        path.join(assetsDir, "manifest.json"),
        JSON.stringify(manifest, null, 2),
    );
}

transformManifest();

// Copy static files to assets/
const rootFiles = fs.readdirSync(__dirname);
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
esbuild.build({
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
    minify: false,
    sourcemap: isDevelopment ? "inline" : false,
    target: "es2021",
    banner: {
        js: `
var __current_script_src = typeof document !== 'undefined' && document.currentScript ? document.currentScript.src : undefined;
if (!('__import_meta_url' in globalThis)) {
    Object.defineProperty(globalThis, '__import_meta_url', {
        get: function() {
            return globalThis.__rufflePublicPath__ ||
                   __current_script_src ||
                   (typeof location !== 'undefined' ? location.href : '');
        },
        configurable: true,
        enumerable: true
    });
}
`.trim(),
    },
    define: {
        "process.env.NODE_ENV": JSON.stringify(mode),
        "import.meta.url": "globalThis.__import_meta_url",
    },
});

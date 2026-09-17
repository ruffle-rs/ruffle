import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import crypto from "node:crypto";
import { fileURLToPath, URL } from "node:url";
import json5 from "json5";
import * as esbuild from "esbuild";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const distDir = path.join(__dirname, "dist");
const coreDir = path.resolve(__dirname, "../core");

// 1. Clean dist directory
if (fs.existsSync(distDir)) {
    fs.rmSync(distDir, { recursive: true, force: true });
}
fs.mkdirSync(distDir, { recursive: true });

// 2. Transform and emit package.json from npm-package.json5
const rawPackageJson5 = fs.readFileSync(
    path.join(__dirname, "npm-package.json5"),
    "utf8",
);
const pkg = json5.parse(rawPackageJson5);
pkg.version = process.env.npm_package_version || "0.0.0";
fs.writeFileSync(
    path.join(distDir, "package.json"),
    JSON.stringify(pkg, null, 2),
);

// 3. Copy static files (LICENSE*, README.md)
const rootFiles = fs.readdirSync(__dirname);
for (const file of rootFiles) {
    if (file.startsWith("LICENSE") || file === "README.md") {
        fs.copyFileSync(path.join(__dirname, file), path.join(distDir, file));
    }
}

// 4. Hash and copy WASM binaries
function copyAndHashWasm(filename) {
    const sourcePath = path.join(coreDir, "dist", filename);

    if (!fs.existsSync(sourcePath)) {
        throw new Error(`Could not find ${sourcePath}`);
    }

    const buffer = fs.readFileSync(sourcePath);
    const hash = crypto
        .createHash("md5")
        .update(buffer)
        .digest("hex")
        .slice(0, 20);

    const ext = path.extname(filename);
    const base = path.basename(filename, ext);
    const hashedName = `${base}.${hash}${ext}`;

    fs.writeFileSync(path.join(distDir, hashedName), buffer);

    return `./${hashedName}`;
}

const isDualWasm = process.env.BUILD_WASM_MVP === "true";

const wasmExtPath = copyAndHashWasm("ruffle_web_bg.wasm");

if (!wasmExtPath) {
    throw new Error("Could not find core/dist/ruffle_web_bg.wasm");
}

const wasmMvpPath = isDualWasm
    ? copyAndHashWasm("ruffle_web-wasm_mvp_bg.wasm")
    : wasmExtPath;

// 5. Bundle with esbuild
const isProduction = process.env.NODE_ENV !== "development";

// This handles rewriting static original WASM URLs to their hashed counterparts
const wasmUrlPlugin = {
    name: "rewrite-wasm-urls",

    setup(build) {
        build.onLoad({ filter: /\.js$/ }, async (args) => {
            // Only modify JavaScript coming from core/dist.
            if (!args.path.startsWith(path.join(coreDir, "dist"))) {
                return;
            }

            let contents = await fs.promises.readFile(args.path, "utf8");

            contents = contents
                .replace(
                    /new URL\(["']\.?\/?ruffle_web_bg\.wasm["'],\s*import\.meta\.url\)/g,
                    `new URL(${JSON.stringify(wasmExtPath)}, import.meta.url)`,
                )
                .replace(
                    /new URL\(["']\.?\/?ruffle_web-wasm_mvp_bg\.wasm["'],\s*import\.meta\.url\)/g,
                    `new URL(${JSON.stringify(wasmMvpPath)}, import.meta.url)`,
                );

            return {
                contents,
                loader: "js",
            };
        });
    },
};

await esbuild.build({
    entryPoints: [path.join(__dirname, "js/ruffle.js")],
    bundle: true,
    outdir: distDir,
    entryNames: "ruffle",
    format: "iife",
    minify: isProduction,
    sourcemap: true,
    charset: "ascii",
    target: "es2015",
    // esbuild bundles load-ruffle into ruffle.js, so import.meta.url would
    // otherwise refer to the bundle rather than the dynamically configured
    // self-hosted asset path.
    banner: {
        js: `
Object.defineProperty(typeof globalThis !== "undefined" ? globalThis : typeof window !== "undefined" ? window : self, "__ruffle_public_path", {
    get: function() {
        return (typeof window !== "undefined" && window.__ruffle_public_path__) ||
               (typeof document !== "undefined" && document.currentScript && document.currentScript.src) ||
               (typeof location !== "undefined" ? location.href : "");
    },
    configurable: true
    });
`.trim(),
    },
    define: {
        "process.env.NODE_ENV": JSON.stringify(
            process.env.NODE_ENV || "production",
        ),
        "import.meta.url": "__ruffle_public_path",
    },
    plugins: [wasmUrlPlugin],
});

console.log("ESBuild complete!");

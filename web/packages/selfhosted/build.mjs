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
let wasmExtPath = "./ruffle_web_bg.wasm";
let wasmMvpPath = "./ruffle_web-wasm_mvp_bg.wasm";

function copyAndHashWasm(dir) {
    if (!fs.existsSync(dir)) {
        return;
    }
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory() && entry.name !== "node_modules") {
            copyAndHashWasm(fullPath);
        } else if (entry.isFile() && entry.name.endsWith(".wasm")) {
            const buffer = fs.readFileSync(fullPath);
            const hash = crypto
                .createHash("md5")
                .update(buffer)
                .digest("hex")
                .slice(0, 20);
            const ext = path.extname(entry.name);
            const base = path.basename(entry.name, ext);
            const hashedName = `${base}.${hash}${ext}`;

            fs.writeFileSync(path.join(distDir, hashedName), buffer);

            if (entry.name.includes("mvp")) {
                wasmMvpPath = `./${hashedName}`;
            } else {
                wasmExtPath = `./${hashedName}`;
            }
        }
    }
}
copyAndHashWasm(coreDir);

// 5. Bundle with esbuild
const isProduction = process.env.NODE_ENV !== "development";

await esbuild.build({
    entryPoints: [path.join(__dirname, "js/ruffle.js")],
    bundle: true,
    outdir: distDir,
    entryNames: "ruffle",
    format: "iife",
    minify: isProduction,
    sourcemap: true,
    charset: "ascii",
    target: ["es2021", "chrome90", "firefox90", "safari15"],
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
        __WASM_EXT_PATH__: JSON.stringify(wasmExtPath),
        __WASM_MVP_PATH__: JSON.stringify(wasmMvpPath),
    },
});

console.log("ESBuild complete!");

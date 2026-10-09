import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, URL } from "node:url";
import json5 from "json5";
import * as esbuild from "esbuild";
import { wasmAssetsPlugin } from "../core/tools/esbuild_wasm_assets.ts";

const __dirname: string = fileURLToPath(new URL(".", import.meta.url));
const distDir: string = path.join(__dirname, "dist");
const coreDir: string = path.resolve(__dirname, "../core");

// 1. Clean dist directory
fs.rmSync(distDir, { recursive: true, force: true });
fs.mkdirSync(distDir, { recursive: true });

// 2. Transform and emit package.json from npm-package.json5
const rawPackageJson5: string = fs.readFileSync(
    path.join(__dirname, "npm-package.json5"),
    "utf8",
);

interface PackageJson {
    version: string;
    [key: string]: unknown;
}

const pkg: PackageJson = json5.parse(rawPackageJson5);

const pkgVersion = process.env["npm_package_version"];

if (pkgVersion === undefined) {
    throw new Error("npm_package_version environment variable is not defined");
}

pkg.version = pkgVersion;
fs.writeFileSync(
    path.join(distDir, "package.json"),
    JSON.stringify(pkg, null, 2),
);

// 3. Copy static files (LICENSE*, README.md)
const rootFiles: string[] = fs.readdirSync(__dirname);
for (const file of rootFiles) {
    if (file.startsWith("LICENSE") || file === "README.md") {
        fs.copyFileSync(path.join(__dirname, file), path.join(distDir, file));
    }
}

// 4. Bundle with esbuild
const isProduction: boolean = process.env["NODE_ENV"] !== "development";

await esbuild.build({
    entryPoints: [path.join(__dirname, "js/ruffle.ts")],
    bundle: true,
    outdir: distDir,
    entryNames: "ruffle",
    format: "iife",
    assetNames: "[name].[hash]",
    // Make our code and dependencies use built-ins that the page can't break.
    inject: [path.join(coreDir, "dist", "pristine-globals.js")],
    minify: isProduction,
    sourcemap: true,
    target: "es2021",
    plugins: [wasmAssetsPlugin()],
});

console.log("ESBuild complete!");

import { execFileSync } from "child_process";
import { copyFileSync, mkdirSync, rmSync } from "fs";
import * as process from "process";

function runWasmOpt({ path, flags }: { path: string; flags?: string[] }) {
    let args = ["-o", path, "-O", "-g", path];
    if (flags) {
        args = args.concat(flags);
    }
    execFileSync("wasm-opt", args, {
        stdio: "inherit",
    });
}
function runWasmBindgen({
    path,
    outName,
    flags,
    dir,
}: {
    path: string;
    outName: string;
    flags?: string[];
    dir: string;
}) {
    let args = [
        path,
        "--target",
        "web",
        "--out-dir",
        dir,
        "--out-name",
        outName,
    ];
    if (flags) {
        args = args.concat(flags);
    }
    execFileSync("wasm-bindgen", args, {
        stdio: "inherit",
    });
}
function cargoBuild({
    profile,
    features,
    rustFlags,
}: {
    profile?: string;
    features?: string[];
    rustFlags?: string[];
}) {
    let args = ["build", "--locked", "--target", "wasm32-unknown-unknown"];
    if (profile) {
        args.push("--profile", profile);
    }
    if (process.env["CARGO_FEATURES"]) {
        features = (features || []).concat(
            process.env["CARGO_FEATURES"].split(","),
        );
    }
    if (features) {
        args.push("--features", features.join(","));
    }
    let totalRustFlags = process.env["RUSTFLAGS"] || "";
    if (rustFlags) {
        if (totalRustFlags) {
            totalRustFlags += ` ${rustFlags.join(" ")}`;
        } else {
            totalRustFlags = rustFlags.join(" ");
        }
    }
    if (process.env["CARGO_FLAGS"]) {
        args = args.concat(process.env["CARGO_FLAGS"].split(" "));
    }
    execFileSync("cargo", args, {
        env: Object.assign(Object.assign({}, process.env), {
            RUSTFLAGS: totalRustFlags,
        }),
        stdio: "inherit",
    });
}
function buildWasm(
    profile: string,
    filename: string,
    optimise: boolean,
    extensions: boolean,
    wasmSource: string,
) {
    const rustFlags = ["--cfg=web_sys_unstable_apis", "-Aunknown_lints"];
    const wasmBindgenFlags = [];
    const wasmOptFlags = [];
    const flavor = extensions ? "extensions" : "vanilla";
    if (extensions) {
        rustFlags.push(
            "-C",
            "target-feature=+bulk-memory,+simd128,+nontrapping-fptoint,+sign-ext,+reference-types",
        );
        wasmBindgenFlags.push("--reference-types");
        wasmOptFlags.push("--enable-reference-types");
    }
    let originalWasmPath;
    if (wasmSource === "cargo" || wasmSource === "cargo_and_store") {
        console.log(`Building ${flavor} with cargo...`);
        cargoBuild({
            profile,
            rustFlags,
        });
        originalWasmPath = `../../../target/wasm32-unknown-unknown/${profile}/ruffle_web.wasm`;
        if (wasmSource === "cargo_and_store") {
            copyFileSync(originalWasmPath, `../../dist/${filename}.wasm`);
        }
    } else if (wasmSource === "existing") {
        originalWasmPath = `../../dist/${filename}.wasm`;
    } else {
        throw new Error(
            "Invalid wasm source: must be one of 'cargo', 'cargo_and_store' or 'existing'",
        );
    }
    console.log(`Running wasm-bindgen on ${flavor}...`);
    runWasmBindgen({
        path: originalWasmPath,
        outName: filename,
        dir: "dist",
        flags: wasmBindgenFlags,
    });
    if (optimise) {
        console.log(`Running wasm-opt on ${flavor}...`);
        runWasmOpt({
            path: `dist/${filename}_bg.wasm`,
            flags: wasmOptFlags,
        });
    }
}
function copyStandIn(from: string, to: string) {
    const suffixes = [`_bg.wasm`, `_bg.wasm.d.ts`, `.js`, `.d.ts`];
    console.log(`Copying ${from} as a stand-in for ${to}...`);
    for (const suffix of suffixes) {
        copyFileSync(`dist/${from}${suffix}`, `dist/${to}${suffix}`);
    }
}
function detectWasmOpt() {
    try {
        execFileSync("wasm-opt", ["--version"]);
        return true;
    } catch (_a) {
        return false;
    }
}
const buildExtensions = !!process.env["ENABLE_WASM_EXTENSIONS"];
const wasmSource = process.env["WASM_SOURCE"] || "cargo";
const hasWasmOpt = detectWasmOpt();
if (!hasWasmOpt) {
    console.log(
        "NOTE: Since wasm-opt could not be found (or it failed), the resulting module might not perform that well, but it should still work.",
    );
}
if (wasmSource === "cargo_and_store") {
    rmSync("../../dist", { recursive: true, force: true });
    mkdirSync("../../dist");
}
buildWasm("web-vanilla-wasm", "ruffle_web", hasWasmOpt, false, wasmSource);
if (buildExtensions) {
    buildWasm(
        "web-wasm-extensions",
        "ruffle_web-wasm_extensions",
        hasWasmOpt,
        true,
        wasmSource,
    );
} else {
    copyStandIn("ruffle_web", "ruffle_web-wasm_extensions");
}

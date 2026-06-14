import { execFileSync } from "child_process";
import { copyFileSync, mkdirSync, rmSync } from "fs";
import * as process from "process";
// TEMP wasm64 vendoring shim — remove when upstream crates support wasm64.
import { ensureWasm64Vendored } from "./vendor_wasm64";

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
    extensions,
}: {
    profile?: string;
    features?: string[];
    rustFlags?: string[];
    extensions?: boolean;
}) {
    let args = ["build", "--locked", "--target", "wasm32-unknown-unknown"];
    if (!extensions) {
        args.push("-Z");
        args.push("build-std=std,panic_abort");
    }

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
    const env: NodeJS.ProcessEnv = {
        ...process.env,
        RUSTFLAGS: totalRustFlags,
        RUSTC_BOOTSTRAP: extensions ? "0" : "1",
    };

    if (!extensions) {
        // C dependencies (e.g. jpegxr's jxrlib) are compiled by cc-rs/clang,
        // whose default wasm32 target enables post-MVP features like
        // reference-types and multivalue. Those bits end up in the linked
        // module's target_features section, which makes wasm-bindgen attempt
        // externref transforms and then fail with "failed to find the
        // __wbindgen_externref_table_dealloc function". Force clang to MVP too.
        // See: https://github.com/ruffle-rs/ruffle/issues/23751
        // and: https://github.com/wasm-bindgen/wasm-bindgen/issues/4654
        env["CFLAGS_wasm32_unknown_unknown"] ??= "";
        env["CFLAGS_wasm32_unknown_unknown"] +=
            " -mno-reference-types -mno-multivalue";
    }

    execFileSync("cargo", args, { env, stdio: "inherit" });
}
function buildWasm(
    profile: string,
    filename: string,
    optimise: boolean,
    extensions: boolean,
    wasmSource: string,
) {
    const rustFlags = [
        "--cfg=web_sys_unstable_apis",
        '--cfg=getrandom_backend="wasm_js"',
        "-Aunknown_lints",
    ];
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
    } else {
        rustFlags.push("-C", "target-cpu=mvp");
    }
    let originalWasmPath;
    if (wasmSource === "cargo" || wasmSource === "cargo_and_store") {
        console.log(`Building ${flavor} with cargo...`);
        cargoBuild({
            profile,
            rustFlags,
            extensions,
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
function buildWasm64(
    filename: string,
    optimise: boolean,
    wasmSource: string,
) {
    // Experimental Memory64 (wasm64-unknown-unknown) build, gated by BUILD_WASM64.
    // - Tier-3 target => needs `-Z build-std` (+ RUSTC_BOOTSTRAP to allow it on stable).
    // - getrandom's `wasm_js` backend is upstream-gated to wasm32 only, so we use its
    //   `custom` backend (the symbol is provided by web/src/getrandom_custom.rs on wasm64).
    // - wgpu/wgpu-core/wgpu-hal/wgpu-types/wgpu-core-deps-wasm/glow/rfd are vendored with
    //   their cfg gates widened from target_arch="wasm32" to target_family="wasm", wired via
    //   [patch.crates-io] in the root Cargo.toml.
    // Java must be on PATH (playerglobal build), as for the normal build.
    const target = "wasm64-unknown-unknown";
    const profile = "web-wasm-extensions";
    const rustFlags = [
        '--cfg=getrandom_backend="custom"',
        "--cfg=web_sys_unstable_apis",
        "-Aunknown_lints",
    ];
    let originalWasmPath;
    if (wasmSource === "cargo" || wasmSource === "cargo_and_store") {
        console.log("Building wasm64 (Memory64) with cargo...");
        let totalRustFlags = process.env["RUSTFLAGS"] || "";
        totalRustFlags = totalRustFlags
            ? `${totalRustFlags} ${rustFlags.join(" ")}`
            : rustFlags.join(" ");
        const env: NodeJS.ProcessEnv = {
            ...process.env,
            RUSTFLAGS: totalRustFlags,
            RUSTC_BOOTSTRAP: "1",
        };
        execFileSync(
            "cargo",
            [
                "build",
                "--locked",
                "--target",
                target,
                "-Z",
                "build-std=std,panic_abort",
                "--profile",
                profile,
                "--package",
                "ruffle_web",
            ],
            { env, stdio: "inherit" },
        );
        originalWasmPath = `../../../target/${target}/${profile}/ruffle_web.wasm`;
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
    console.log("Running wasm-bindgen on wasm64...");
    runWasmBindgen({
        path: originalWasmPath,
        outName: filename,
        dir: "dist",
        flags: [],
    });
    if (optimise) {
        console.log("Running wasm-opt on wasm64...");
        runWasmOpt({
            path: `dist/${filename}_bg.wasm`,
            flags: ["--enable-memory64"],
        });
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
const buildWasmMvp = !!process.env["BUILD_WASM_MVP"];
const buildWasm64Flag = !!process.env["BUILD_WASM64"];
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
if (buildWasm64Flag) {
    // Fetch+patch the vendored crates and add the [patch.crates-io] block to
    // the root Cargo.toml. Wasm32 / native builds never touch any of this:
    // the committed Cargo.toml has no patch block, no vendor/ paths to
    // resolve, and this script does not run.
    ensureWasm64Vendored();
    // Memory64 build: produce `ruffle_web` (the module the loader imports when
    // wasm-extensions are detected — engines passing that check generally also support Memory64).
    buildWasm64("ruffle_web", hasWasmOpt, wasmSource);
} else {
    buildWasm("web-wasm-extensions", "ruffle_web", hasWasmOpt, true, wasmSource);
    if (buildWasmMvp) {
        buildWasm(
            "web-wasm-mvp",
            "ruffle_web-wasm_mvp",
            hasWasmOpt,
            false,
            wasmSource,
        );
    }
}

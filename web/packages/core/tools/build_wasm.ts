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
function detectWasmOpt() {
    try {
        execFileSync("wasm-opt", ["--version"]);
        return true;
    } catch (_a) {
        return false;
    }
}
const buildWasmMvp = !!process.env["BUILD_WASM_MVP"];
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

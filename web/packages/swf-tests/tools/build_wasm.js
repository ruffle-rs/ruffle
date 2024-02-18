import { execFileSync } from "child_process";

import process from "process";

function runWasmOpt({ path, flags }) {
    let args = ["-o", path, "-O", "-g", path];
    if (flags) {
        args = args.concat(flags);
    }
    execFileSync("wasm-opt", args, {
        stdio: "inherit",
    });
}
function runWasmBindgen({ path, outName, flags, dir }) {
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
function cargoBuild({ profile, features, rustFlags }) {
    let args = [
        "build",
        "--locked",
        "--target",
        "wasm32-unknown-unknown",
        "-p",
        "web_test_runner",
    ];
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
function cargoClean() {
    execFileSync("cargo", ["clean"], {
        stdio: "inherit",
    });
}
function buildWasm(profile, filename, optimise, extensions) {
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
    console.log(`Building ${flavor} with cargo...`);
    cargoBuild({
        profile,
        rustFlags,
    });
    console.log(`Running wasm-bindgen on ${flavor}...`);
    runWasmBindgen({
        path: `../../../target/wasm32-unknown-unknown/${profile}/web_test_runner.wasm`,
        outName: filename,
        dir: "build",
        flags: wasmBindgenFlags,
    });
    if (process.env["ENABLE_CARGO_CLEAN"]) {
        console.log(`Running cargo clean...`);
        cargoClean();
    }
    if (optimise) {
        console.log(`Running wasm-opt on ${flavor}...`);
        runWasmOpt({
            path: `build/${filename}_bg.wasm`,
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
const hasWasmOpt = detectWasmOpt();
if (!hasWasmOpt) {
    console.log(
        "NOTE: Since wasm-opt could not be found (or it failed), the resulting module might not perform that well, but it should still work.",
    );
}
buildWasm("web-vanilla-wasm", "web_test_runner", hasWasmOpt, false);

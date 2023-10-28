const { execFileSync } = require("child_process");
const { copyFileSync } = require("fs");
const process = require("process");

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
        path: `../../../target/wasm32-unknown-unknown/${profile}/ruffle_web.wasm`,
        outName: filename,
        dir: "dist",
        flags: wasmBindgenFlags,
    });
    if (process.env["ENABLE_CARGO_CLEAN"]) {
        console.log(`Running cargo clean...`);
        cargoClean();
    }
    if (optimise) {
        console.log(`Running wasm-opt on ${flavor}...`);
        runWasmOpt({
            path: `dist/${filename}_bg.wasm`,
            flags: wasmOptFlags,
        });
    }
}
function copyStandIn(from, to) {
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
const hasWasmOpt = detectWasmOpt();
if (!hasWasmOpt) {
    console.log(
        "NOTE: Since wasm-opt could not be found (or it failed), the resulting module might not perform that well, but it should still work.",
    );
}
buildWasm("web-vanilla-wasm", "ruffle_web", hasWasmOpt, false);
if (buildExtensions) {
    buildWasm(
        "web-wasm-extensions",
        "ruffle_web-wasm_extensions",
        hasWasmOpt,
        true,
    );
} else {
    copyStandIn("ruffle_web", "ruffle_web-wasm_extensions");
}

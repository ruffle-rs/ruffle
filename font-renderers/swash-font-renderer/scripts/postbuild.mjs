/**
 * Post-build rewrites for the swash-font-renderer wasm-bindgen output.
 *
 * wasm-bindgen emits `<name>_bg.wasm` plus a JS glue file that defaults
 * to exporting `__wbg_init`. For this pluggable bridge we want:
 *
 *   1. The wasm artefact to be called `<name>.wasm`, dropping the
 *      `_bg` suffix so the filename matches the JS sibling.
 *   2. The JS glue to self-initialize the wasm at module load via
 *      top-level await, and expose the FontBridge object as the
 *      default export - so that a consumer can write simply:
 *
 *          import swashRenderer from "./swash-font-renderer.js";
 *          globalThis.__ruffleCustomFontRenderer = swashRenderer;
 *          window.RufflePlayer.config = {
 *              deviceFontRenderer: "custom",
 *              // ...
 *          };
 *
 *      with no explicit `await init()` call. Ruffle looks up the
 *      bridge lazily from the global slot on each font request, so
 *      this module can finish loading after the player has already
 *      been created.
 *
 * The script is idempotent: re-running it on an already-patched
 * directory is a no-op.
 */

import { readFile, writeFile, rename, access } from "node:fs/promises";
import { constants } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const PKG_DIR = resolve(
    dirname(fileURLToPath(import.meta.url)),
    "..",
    "pkg",
);
const NAME = "swash-font-renderer";

async function exists(path) {
    try {
        await access(path, constants.F_OK);
        return true;
    } catch {
        return false;
    }
}

async function renameIfNeeded(from, to) {
    const fromPath = resolve(PKG_DIR, from);
    const toPath = resolve(PKG_DIR, to);
    if (await exists(fromPath)) {
        await rename(fromPath, toPath);
        console.log(`renamed  ${from} -> ${to}`);
    } else if (!(await exists(toPath))) {
        throw new Error(
            `postbuild: expected either ${from} or ${to} to exist in ${PKG_DIR}`,
        );
    }
}

async function patchGlueJs() {
    const jsPath = resolve(PKG_DIR, `${NAME}.js`);
    let src = await readFile(jsPath, "utf8");

    src = src.replaceAll(`${NAME}_bg.wasm`, `${NAME}.wasm`);

    const oldExport = "export { initSync, __wbg_init as default };";
    const newExport = "export { initSync, __wbg_init };";
    if (src.includes(oldExport)) {
        src = src.replace(oldExport, newExport);
    } else if (!src.includes(newExport)) {
        throw new Error(
            `postbuild: could not locate the wasm-bindgen default export line in ${NAME}.js`,
        );
    }

    const marker = "/* ruffle-postbuild: auto-init + bridge default export */";
    if (!src.includes(marker)) {
        src += `\n${marker}\nawait __wbg_init();\nexport default { createRenderer, registerFontData };\n`;
    }

    await writeFile(jsPath, src);
    console.log(`patched  ${NAME}.js (auto-init + default bridge export)`);
}

async function patchGlueDts() {
    const dtsPath = resolve(PKG_DIR, `${NAME}.d.ts`);
    let src = await readFile(dtsPath, "utf8");

    src = src.replaceAll(`${NAME}_bg.wasm`, `${NAME}.wasm`);

    const defaultDecl =
        /export default function __wbg_init \([\s\S]*?\): Promise<InitOutput>;\s*$/m;
    const marker = "/* ruffle-postbuild: bridge default export */";
    if (defaultDecl.test(src)) {
        src = src.replace(
            defaultDecl,
            [
                "export function __wbg_init(",
                "    module_or_path?:",
                "        | { module_or_path: InitInput | Promise<InitInput> }",
                "        | InitInput",
                "        | Promise<InitInput>,",
                "): Promise<InitOutput>;",
                "",
                marker,
                "declare const bridge: {",
                "    createRenderer: typeof createRenderer;",
                "    registerFontData: typeof registerFontData;",
                "};",
                "export default bridge;",
            ].join("\n"),
        );
    } else if (!src.includes(marker)) {
        throw new Error(
            `postbuild: could not locate the default export declaration in ${NAME}.d.ts`,
        );
    }

    await writeFile(dtsPath, src);
    console.log(`patched  ${NAME}.d.ts (bridge default export)`);
}

await renameIfNeeded(`${NAME}_bg.wasm`, `${NAME}.wasm`);
await renameIfNeeded(`${NAME}_bg.wasm.d.ts`, `${NAME}.wasm.d.ts`);
await patchGlueJs();
await patchGlueDts();

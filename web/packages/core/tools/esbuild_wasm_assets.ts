import fs from "node:fs";
import type * as esbuild from "esbuild";

/**
 * Makes esbuild emit the .wasm files referenced by ruffle-core's load-ruffle.js.
 *
 * ES module bundlers pick them up from `new URL("./x.wasm", import.meta.url)`,
 * but esbuild doesn't support that pattern. Instead, this turns each .wasm
 * string literal in load-ruffle.js into an import handled by the "file"
 * loader, so esbuild copies the referenced files (named after `assetNames`)
 * and substitutes their paths, relative to the output directory.
 *
 * It also defines `import.meta.url` as undefined (unless the build defines it
 * itself), so that the ES module branch of load-ruffle.js is dropped.
 *
 * @returns The esbuild plugin.
 */
export function wasmAssetsPlugin(): esbuild.Plugin {
    return {
        name: "wasm-assets",

        setup(build: esbuild.PluginBuild) {
            build.initialOptions.loader = {
                ...build.initialOptions.loader,
                ".wasm": "file",
            };
            build.initialOptions.define = {
                // Lets esbuild drop load-ruffle.js's ES module branch as dead code.
                "import.meta.url": "undefined",
                ...build.initialOptions.define,
            };

            let rewroteWasmUrls = false;

            build.onLoad(
                { filter: /(?:^|[/\\])load-ruffle\.js$/ },
                async (args: esbuild.OnLoadArgs) => {
                    let contents: string = await fs.promises.readFile(
                        args.path,
                        "utf8",
                    );

                    const imports = new Map<string, string>();
                    contents = contents.replace(
                        /(["'])(\.\/[^"']+\.wasm)\1/g,
                        (_match: string, _quote: string, specifier: string) => {
                            let id = imports.get(specifier);
                            if (id === undefined) {
                                id = `__ruffleWasm${imports.size}`;
                                imports.set(specifier, id);
                            }
                            return id;
                        },
                    );

                    if (imports.size === 0) {
                        throw new Error(
                            `No .wasm references found in ${args.path}`,
                        );
                    }

                    const header = [...imports]
                        .map(
                            ([specifier, id]) =>
                                `import ${id} from ${JSON.stringify(specifier)};`,
                        )
                        .join("\n");

                    rewroteWasmUrls = true;

                    return {
                        contents: `${header}\n${contents}`,
                        loader: "js",
                    };
                },
            );

            build.onEnd((result: esbuild.BuildResult) => {
                if (!rewroteWasmUrls && result.errors.length === 0) {
                    throw new Error(
                        "load-ruffle.js was never loaded through wasm-assets, so no .wasm files were emitted",
                    );
                }
            });
        },
    };
}

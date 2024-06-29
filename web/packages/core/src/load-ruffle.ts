/**
 * Conditional ruffle loader
 */

import {
    bulkMemory,
    simd,
    saturatedFloatToInt,
    signExtensions,
    referenceTypes,
} from "wasm-feature-detect";
import type { RuffleInstanceBuilder, ZipWriter } from "../dist/ruffle_web";
import { setPolyfillsOnLoad } from "./js-polyfills";
import { internalSourceApi } from "./source-api";

type ProgressCallback = (bytesLoaded: number, bytesTotal: number) => void;

/**
 * Load ruffle from an automatically-detected location.
 *
 * This function returns a new instance of Ruffle and downloads it every time.
 * You should not use it directly; this module will memoize the resource
 * download.
 *
 * @param progressCallback The callback that will be run with Ruffle's download progress.
 * @returns A ruffle-builder constructor that may be used to create new RuffleInstanceBuilder
 * instances.
 */
async function fetchRuffle(
    progressCallback?: ProgressCallback,
): Promise<[typeof RuffleInstanceBuilder, typeof ZipWriter]> {
    // Apply some pure JavaScript polyfills to prevent conflicts with external
    // libraries, if needed.
    setPolyfillsOnLoad();

    // NOTE: Keep this list in sync with $RUSTFLAGS in the CI build config!
    const extensionsSupported: boolean = (
        await Promise.all([
            bulkMemory(),
            simd(),
            saturatedFloatToInt(),
            signExtensions(),
            referenceTypes(),
        ])
    ).every(Boolean);

    if (!extensionsSupported) {
        console.log(
            "Some WebAssembly extensions are NOT available, falling back to the vanilla WebAssembly module",
        );
    }

    // Easy "on first load": just set it to something else after the call.
    internalSourceApi.options.onFirstLoad?.();
    internalSourceApi.options.onFirstLoad = () => {};

    // Note: The argument passed to import() has to be a simple string literal,
    // otherwise some bundler will get confused and won't include the module?
    const {
        default: init,
        RuffleInstanceBuilder,
        ZipWriter,
    } = await (extensionsSupported
        ? import("../dist/ruffle_web-wasm_extensions")
        : import("../dist/ruffle_web"));
    let response;
    const wasmUrl = extensionsSupported
        ? new URL("../dist/ruffle_web-wasm_extensions_bg.wasm", import.meta.url)
        : new URL("../dist/ruffle_web_bg.wasm", import.meta.url);
    const wasmResponse = await fetch(wasmUrl);
    // The Pale Moon browser lacks full support for ReadableStream.
    // However, ReadableStream itself is defined.
    const readableStreamProperlyDefined =
        typeof ReadableStreamDefaultController === "function";
    if (progressCallback && readableStreamProperlyDefined) {
        const contentLength =
            wasmResponse?.headers?.get("content-length") || "";
        let bytesLoaded = 0;
        // Use parseInt rather than Number so the empty string is coerced to NaN instead of 0
        const bytesTotal = parseInt(contentLength);
        response = new Response(
            new ReadableStream({
                async start(controller) {
                    const reader = wasmResponse.body?.getReader();
                    if (!reader) {
                        throw "Response had no body";
                    }
                    progressCallback(bytesLoaded, bytesTotal);
                    for (;;) {
                        const { done, value } = await reader.read();
                        if (done) {
                            break;
                        }
                        if (value?.byteLength) {
                            bytesLoaded += value?.byteLength;
                        }
                        controller.enqueue(value);
                        progressCallback(bytesLoaded, bytesTotal);
                    }
                    controller.close();
                },
            }),
            wasmResponse,
        );
    } else {
        response = wasmResponse;
    }

    await init(response);

    return [RuffleInstanceBuilder, ZipWriter];
}

let nativeConstructors: Promise<
    [typeof RuffleInstanceBuilder, typeof ZipWriter]
> | null = null;

/**
 * Obtain an instance of `Ruffle`.
 *
 * This function returns a promise which yields a new `RuffleInstanceBuilder` asynchronously.
 *
 * @param progressCallback The callback that will be run with Ruffle's download progress.
 * @returns A ruffle instance builder.
 */
export async function createRuffleBuilder(
    progressCallback?: ProgressCallback,
): Promise<[RuffleInstanceBuilder, () => ZipWriter]> {
    if (nativeConstructors === null) {
        nativeConstructors = fetchRuffle(progressCallback);
    }

    const constructors = await nativeConstructors;
    return [new constructors[0](), () => new constructors[1]()];
}

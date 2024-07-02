export enum PanicError {
    Unknown,
    CSPConflict,
    FileProtocol,
    InvalidWasm,
    JavascriptConfiguration,
    JavascriptConflict,
    WasmCors,
    WasmDownload,
    WasmMimeType,
    WasmNotFound,
    WasmDisabledMicrosoftEdge,
    InvalidSwf,
    SwfFetchError,
    SwfCors,
}

export class LoadSwfError extends Error {
    readonly #swfUrl: URL | undefined;

    constructor(swfUrl: URL | undefined) {
        super(`Failed to fetch ${swfUrl}`);
        this.#swfUrl = swfUrl;
    }

    get ruffleIndexError() {
        if (this.#swfUrl && !this.#swfUrl.protocol.includes("http")) {
            return PanicError.FileProtocol;
        } else if (
            window.location.origin === this.#swfUrl?.origin ||
            // The extension's internal player page is not restricted by CORS
            window.location.protocol.includes("extension")
        ) {
            return PanicError.SwfFetchError;
        } else {
            // This is a selfhosted build of Ruffle that tried to make a cross-origin request
            return PanicError.SwfCors;
        }
    }
}

export class InvalidSwfError extends Error {
    constructor(swfUrl: URL | undefined) {
        super(`Not a valid swf: ${swfUrl}`);
    }

    get ruffleIndexError() {
        return PanicError.InvalidSwf;
    }
}

export class LoadRuffleWasmError extends Error {
    constructor(public cause: Error) {
        super("Failed to load Ruffle WASM");
    }

    get ruffleIndexError() {
        // Serious duck typing. In error conditions, let's not make assumptions.
        if (window.location.protocol === "file:") {
            return PanicError.FileProtocol;
        } else {
            const message = String(this.cause.message).toLowerCase();
            if (message.includes("mime")) {
                return PanicError.WasmMimeType;
            } else if (
                message.includes("networkerror") ||
                message.includes("failed to fetch")
            ) {
                return PanicError.WasmCors;
            } else if (message.includes("disallowed by embedder")) {
                return PanicError.CSPConflict;
            } else if (this.cause.name === "CompileError") {
                return PanicError.InvalidWasm;
            } else if (
                message.includes("could not download wasm module") &&
                this.cause.name === "TypeError"
            ) {
                return PanicError.WasmDownload;
            } else if (this.cause.name === "TypeError") {
                return PanicError.JavascriptConflict;
            } else if (
                navigator.userAgent.includes("Edg") &&
                message.includes("webassembly is not defined")
            ) {
                // Microsoft Edge detection.
                return PanicError.WasmDisabledMicrosoftEdge;
            } else {
                return PanicError.WasmNotFound;
            }
        }
    }
}

export class InvalidOptionsError extends Error {
    constructor(message: string) {
        super(`Invalid options: ${message}`);
    }

    get ruffleIndexError() {
        return PanicError.JavascriptConfiguration;
    }
}

export function getRuffleIndexError(error: Error | null): PanicError {
    if (
        error instanceof InvalidOptionsError ||
        error instanceof InvalidSwfError ||
        error instanceof LoadRuffleWasmError ||
        error instanceof LoadSwfError
    ) {
        return error.ruffleIndexError;
    }
    return PanicError.Unknown;
}

export class LoadSwfError extends Error {
    constructor(
        public swfUrl: URL | undefined,
        public statusNotOk: boolean,
    ) {
        super(`Failed to fetch ${swfUrl}`);
        this.swfUrl = swfUrl;
        this.statusNotOk = statusNotOk;
    }
}

export class InvalidSwfError extends Error {
    constructor(swfUrl: URL | undefined) {
        super(`Not a valid swf: ${swfUrl}`);
    }
}

export class LoadRuffleWasmError extends Error {
    constructor(public cause: Error) {
        super("Failed to load Ruffle WASM");
    }
}

export class InvalidOptionsError extends Error {
    constructor(message: string) {
        super(`Invalid options: ${message}`);
    }
}

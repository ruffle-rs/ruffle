import type { Services } from "@wdio/types";

const capabilities: WebdriverIO.Capabilities[] = [];
const services: Services.ServiceEntry[] = [];

const headless = process.argv.includes("--headless");
const chrome = process.argv.includes("--chrome");
const firefox = process.argv.includes("--firefox");
const edge = process.argv.includes("--edge");
const maxInstances = process.argv.includes("--parallel") ? 4 : 1;

if (chrome) {
    const args = ["--disable-gpu", "--enable-unsafe-swiftshader"];
    if (headless) {
        args.push("--headless");
    }
    capabilities.push({
        "wdio:maxInstances": maxInstances,
        browserName: "chrome",
        "goog:chromeOptions": {
            args,
        },
    });
}

if (edge) {
    const args = ["--disable-gpu", "--enable-unsafe-swiftshader"];
    if (headless) {
        args.push("--headless", "--no-sandbox");
    }
    capabilities.push({
        "wdio:maxInstances": maxInstances,
        browserName: "MicrosoftEdge",
        "ms:edgeOptions": {
            args,
        },
    });
}

if (firefox) {
    const args = [];
    if (headless) {
        args.push("-headless");
    }
    capabilities.push({
        "wdio:maxInstances": maxInstances,
        browserName: "firefox",
        "moz:firefoxOptions": {
            args,
        },
    });
}

services.push([
    "static-server",
    {
        folders: [
            { mount: "/dist", path: "./dist" },
            { mount: "/test_assets", path: "./test_assets" },
            { mount: "/test", path: "./test" },
        ],
        port: 4567,
    },
]);

declare global {
    interface Console {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        logs: { level: string; message: any[] }[];
    }
}

export const config: WebdriverIO.Config = {
    runner: "local",
    specs: [
        "./test/polyfill/**/test.ts",
        "./test/js_api/*.ts",
        "./test/integration_tests/**/test.ts",
    ],
    maxInstances: maxInstances,
    capabilities,
    // wdio produces a lot of spam on info
    logLevel: "warn",
    bail: 0,
    baseUrl: "http://localhost",
    waitforTimeout: 30000,
    connectionRetryTimeout: 120000,
    connectionRetryCount: 3,
    specFileRetries: 2,
    services,
    framework: "mocha",
    reporters: ["spec"],
    mochaOpts: {
        ui: "bdd",
        timeout: 120000,
    },
};

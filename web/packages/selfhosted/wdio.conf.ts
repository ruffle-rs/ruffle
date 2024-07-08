import type { Options, Services } from "@wdio/types";

const capabilities: WebdriverIO.Capabilities[] = [];
const services: Services.ServiceEntry[] = [];

const headless = process.argv.includes("--headless");
const chrome = process.argv.includes("--chrome");

if (chrome) {
    const args = ["--disable-gpu"];
    if (headless) {
        args.push("--headless");
    }
    capabilities.push({
        "wdio:maxInstances": 1,
        browserName: "chrome",
        "goog:chromeOptions": {
            args,
        },
    });
    services.push("chromedriver");
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

export const config: Options.Testrunner = {
    runner: "local",
    specs: [
        "./test/polyfill/**/test.ts",
        "./test/js_api/*.ts",
        "./test/integration_tests/**/test.ts",
    ],
    maxInstances: 10,
    capabilities,
    logLevel: "info",
    bail: 0,
    baseUrl: "http://localhost",
    waitforTimeout: 30000,
    connectionRetryTimeout: 120000,
    connectionRetryCount: 3,
    services,
    framework: "mocha",
    reporters: ["spec"],
    mochaOpts: {
        ui: "bdd",
        timeout: 120000,
    },
};

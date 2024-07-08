import type { Options } from "@wdio/types";

export const config: Options.Testrunner = {
    runner: "local",
    specs: [
        "./test/polyfill/**/test.ts",
        "./test/js_api/*.ts",
        "./test/integration_tests/**/test.ts",
    ],
    maxInstances: 10,
    capabilities: [
        {
            "wdio:maxInstances": 5,
            browserName: "chrome",
            "goog:chromeOptions": {
                args: ["--headless", "--disable-gpu"],
            },
        },
    ],
    logLevel: "info",
    bail: 0,
    baseUrl: "http://localhost",
    waitforTimeout: 30000,
    connectionRetryTimeout: 120000,
    connectionRetryCount: 3,
    services: [
        "chromedriver",
        [
            "static-server",
            {
                folders: [
                    { mount: "/dist", path: "./dist" },
                    { mount: "/test_assets", path: "./test_assets" },
                    { mount: "/test", path: "./test" },
                ],
                port: 4567,
            },
        ],
    ],
    framework: "mocha",
    reporters: ["spec"],
    mochaOpts: {
        ui: "bdd",
        timeout: 120000,
    },
};

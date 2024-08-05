import type { Options, Services } from "@wdio/types";
import { BrowserStackCapabilities } from "@wdio/types/build/Capabilities";

const capabilities: WebdriverIO.Capabilities[] = [];
const services: Services.ServiceEntry[] = [];

const headless = process.argv.includes("--headless");
const chrome = process.argv.includes("--chrome");
const firefox = process.argv.includes("--firefox");
const edge = process.argv.includes("--edge");
const browserstack = process.argv.includes("--browserstack");
const oldVersions = process.argv.includes("--oldVersions");

let user: string | undefined = undefined;
let key: string | undefined = undefined;
let setupLogging = async () => {};
let reportLogging = async () => {};

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
}

if (edge) {
    const args = ["--disable-gpu"];
    if (headless) {
        args.push("--headless");
    }
    capabilities.push({
        "wdio:maxInstances": 1,
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
        "wdio:maxInstances": 1,
        browserName: "firefox",
        "moz:firefoxOptions": {
            args,
        },
    });
}

if (browserstack) {
    user = process.env["BROWSERSTACK_USERNAME"];
    key = process.env["BROWSERSTACK_ACCESS_KEY"];
    if (!user || !key) {
        throw new Error(
            "BROWSERSTACK_USERNAME and BROWSERSTACK_ACCESS_KEY environment variables are required",
        );
    }
    const buildIdentifier =
        process.env["BROWSERSTACK_BUILD_ID"] || crypto.randomUUID();
    const buildName = process.env["BROWSERSTACK_BUILD_NAME"] || buildIdentifier;
    const bsOptions: BrowserStackCapabilities = {
        buildName,
        buildIdentifier,
        projectName: "Ruffle Selfhosted",
        networkLogs: true,
        consoleLogs: "info",
        idleTimeout: 300, // Max time the browser's main thread can be blocked
    };
    services.push([
        "browserstack",
        {
            testObservability: true,
            testObservabilityOptions: {
                projectName: "Ruffle Selfhosted",
                buildName,
            },
            browserstackLocal: true,
        },
    ]);

    // Relatively latest versions for Mobile

    capabilities.push({
        browserName: "Chrome",
        "bstack:options": {
            deviceName: "Google Pixel 8",
            osVersion: "14.0",
            ...bsOptions,
        },
    });
    capabilities.push({
        browserName: "Safari",
        "bstack:options": {
            deviceName: "iPhone 15",
            osVersion: "17",
            deviceOrientation: "portrait",
            ...bsOptions,
        },
        "wdio:exclude": [
            "./test/integration_tests/keyboard_input/test.ts", // Doesn't work on iOS at time of writing
            "./test/polyfill/classic_frames_provided/test.ts", // Flaky on iOS
        ],
    });

    // These are our supposed minimum-supported browsers
    if (oldVersions) {
        capabilities.push({
            browserName: "Chrome",
            "bstack:options": {
                os: "Windows",
                osVersion: "10",
                browserVersion: "87.0",
                ...bsOptions,
            },
        });
        capabilities.push({
            browserName: "Firefox",
            "bstack:options": {
                os: "Windows",
                osVersion: "10",
                browserVersion: "84.0",
                ...bsOptions,
            },
        });
        capabilities.push({
            browserName: "Safari",
            "bstack:options": {
                os: "OS X",
                osVersion: "Big Sur",
                browserVersion: "14.1",
                ...bsOptions,
            },
        });
    }

    setupLogging = async () => {
        await browser.execute(() => {
            if (console.logs === undefined) {
                const log = console.log.bind(console);
                console.log = function (...args) {
                    console.logs.push({ level: "info", message: args });
                    log.apply(console, args);
                };
                const warn = console.warn.bind(console);
                console.warn = function (...args) {
                    console.logs.push({ level: "warn", message: args });
                    warn.apply(console, args);
                };
                const error = console.error.bind(console);
                console.error = function (...args) {
                    console.logs.push({ level: "error", message: args });
                    error.apply(console, args);
                };
            }
            console.logs = [];
        });
    };

    reportLogging = async () => {
        const logs = await browser.execute(() => {
            const logs = console.logs;
            console.logs = [];
            return logs;
        });
        if (logs) {
            for (const log of logs) {
                const message = `Console: ${log.message}`;
                await driver.executeScript(
                    `browserstack_executor: {"action": "annotate", "arguments": {"data":${JSON.stringify(message)},"level": ${JSON.stringify(log.level)}}}`,
                    [],
                );
            }
        }
    };
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

// @ts-expect-error TS2375 Undefined is the same as not specified here
export const config: Options.Testrunner = {
    user,
    key,
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
    specFileRetries: 2,
    services,
    framework: "mocha",
    reporters: ["spec"],
    mochaOpts: {
        ui: "bdd",
        timeout: 120000,
    },

    async beforeTest() {
        await setupLogging();
    },

    async afterTest() {
        await reportLogging();
    },
};

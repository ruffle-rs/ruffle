/* eslint-disable @typescript-eslint/no-explicit-any */

import { use } from "chai";
import chaiHtml from "chai-html";
import { promises as fs } from "fs";
import * as path from "path";
import json5 from "json5";
import * as toml from "smol-toml";
import {
    getAllTraceOutput,
    hideHardwareAccelerationModal,
    openTest,
    setupAndPlay,
    throwIfError,
    waitForPlayerToLoad,
    waitForRuffle,
} from "../utils";
import { Player } from "ruffle-core";
import sharp from "sharp";
import { MovieMetadata } from "ruffle-core/dist/public/player";

const TESTS_DIR = "../../../tests/tests/swfs";
const TESTS_DESCRIPTOR = "test/swf_tests/swf_tests.json5";

use(chaiHtml);

async function readPng(filePath: string): Promise<Image> {
    const image = sharp(filePath);
    const { data, info } = await image
        .raw()
        .ensureAlpha() // ensures RGBA
        .toBuffer({ resolveWithObject: true });

    return {
        width: info.width,
        height: info.height,
        data: Array.from(data),
    };
}

function deconstruct(object: any, prop: string): any | null {
    if (!(prop in object)) {
        return null;
    }
    const val = object[prop];
    // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
    delete object[prop];
    return val;
}

function assertDeconstructedFully(object: any): any {
    if (Object.keys(object).length > 0) {
        throw Error(
            "Expected object to be fully deconstructed, but it was not. " +
                "It usually means that some feature used by a SWF test is not " +
                "implemented by this runner. Left fields: " +
                Object.keys(object),
        );
    }
}

type Trigger = string | number;

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
interface TestDescriptor {}

interface ImageComparison {
    name: string;
    filename: string;
    checks: ImageComparisonCheck[];
}

interface ImageComparisonCheck {
    tolerance: number;
    outliers: number;
}

interface Image {
    width: number;
    height: number;

    // RGBA data
    // Note: wdio does not support serializing Uint8ClampedArray
    data: Array<number>;
}

class SwfTest {
    name: string;
    descriptor: TestDescriptor;
    testDir: string;

    loadSucceeded: boolean = false;

    testToml: any;
    outputTxt?: string;
    inputJson: Array<any> = [];
    inputJsonPos: number = 0;

    browser?: WebdriverIO.Browser;
    player: ChainablePromiseElement;

    metadata?: MovieMetadata;
    frameRate: number = 0;
    frameTime: number = 0;

    numTicks: number = 0;
    iteration: number = 0;

    sampleCount?: number;
    imageComparisons: Map<Trigger, ImageComparison> = new Map();
    images: Map<Trigger, Image | undefined> = new Map();

    constructor(name: string, descriptor: TestDescriptor) {
        this.name = name;
        this.descriptor = descriptor;
        this.testDir = path.join(TESTS_DIR, this.name);
    }

    async configure() {
        const testTomlPath = path.join(this.testDir, "test.toml");
        const inputJsonPath = path.join(this.testDir, "input.json");
        const outputTxtPath = path.join(this.testDir, "output.txt");

        this.testToml = toml.parse(await fs.readFile(testTomlPath, "utf8"));
        this.outputTxt = await fs.readFile(outputTxtPath, "utf8");
        try {
            this.inputJson = json5.parse(
                await fs.readFile(inputJsonPath, "utf8"),
            );
        } catch {
            // no input.json
        }

        await this.readTestToml();
    }

    async readTestToml() {
        const testToml = JSON.parse(JSON.stringify(this.testToml));
        if (!("num_ticks" in testToml)) {
            throw Error("Only 'num_ticks' is supported for now.");
        }

        this.numTicks = deconstruct(testToml, "num_ticks");

        if ("image_comparisons" in testToml) {
            await this.readTestTomlImageComparisons(
                deconstruct(testToml, "image_comparisons"),
            );
        }

        if ("player_options" in testToml) {
            await this.readTestTomlPlayerOptions(
                deconstruct(testToml, "player_options"),
            );
        }

        assertDeconstructedFully(testToml);
    }

    async readTestTomlImageComparisons(imageComparisons: any) {
        for (const name of Object.keys(imageComparisons)) {
            const imageComparisonToml = deconstruct(imageComparisons, name);
            const trigger =
                deconstruct(imageComparisonToml, "trigger") ?? "last_frame";

            const checks: ImageComparisonCheck[] = [];
            checks.push({
                outliers: 0,
                tolerance: 0,
            });

            const imageComparison: ImageComparison = {
                name,
                filename: `${name}.expected.png`,
                checks,
            };

            assertDeconstructedFully(imageComparisonToml);
            this.imageComparisons.set(trigger, imageComparison);
        }

        assertDeconstructedFully(imageComparisons);
    }

    async readTestTomlPlayerOptions(playerOptions: any) {
        if ("with_renderer" in playerOptions) {
            const withRenderer = deconstruct(playerOptions, "with_renderer");
            deconstruct(withRenderer, "optional"); // ignore

            this.sampleCount = deconstruct(withRenderer, "sample_count");
            assertDeconstructedFully(withRenderer);
        }

        assertDeconstructedFully(playerOptions);
    }

    async load(browser: WebdriverIO.Browser, player: ChainablePromiseElement) {
        this.browser = browser;
        this.player = player;

        this.metadata = await this.browser!.execute((playerElement) => {
            const player = playerElement as Player.PlayerElement;
            return player.ruffle().metadata!;
        }, this.player);
        this.frameRate = this.metadata.frameRate;
        this.frameTime = 1000.0 / this.frameRate;
        this.loadSucceeded = true;
    }

    async runInputForTick() {
        while (this.inputJsonPos < this.inputJson.length) {
            const event = JSON.parse(
                JSON.stringify(this.inputJson[this.inputJsonPos++]),
            );
            const eventType = deconstruct(event, "type");

            switch (eventType) {
                case "Wait":
                    break;
                default:
                    throw new Error(`Unsupported event "${eventType}"`);
            }
            assertDeconstructedFully(event);
        }
    }

    async captureImage(trigger: Trigger) {
        const canvas = await this.player.shadow$("canvas");

        const image: Image | undefined = await this.browser!.execute(function (
            canvasElement,
        ): Image | undefined {
            const canvas = canvasElement as HTMLCanvasElement;
            const ctx2d = canvas.getContext("2d");
            if (ctx2d) {
                const imageData = ctx2d.getImageData(
                    0,
                    0,
                    canvas.width,
                    canvas.height,
                );
                return {
                    width: canvas.width,
                    height: canvas.height,
                    data: Array.from(imageData.data),
                };
            }

            const ctxWebgl =
                canvas.getContext("webgl") ?? canvas.getContext("webgl2");
            if (ctxWebgl) {
                const width = canvas.width;
                const height = canvas.height;
                const data = new Uint8ClampedArray(width * height * 4);
                ctxWebgl.readPixels(
                    0,
                    0,
                    width,
                    height,
                    ctxWebgl.RGBA,
                    ctxWebgl.UNSIGNED_BYTE,
                    data,
                );

                return {
                    width: canvas.width,
                    height: canvas.height,
                    data: Array.from(data),
                };
            }

            return undefined;
        }, canvas);

        this.images.set(trigger, image);
    }

    async tick(timestamp: number) {
        await this.browser!.execute(
            (playerElement, timestamp) => {
                const player = playerElement as Player.PlayerElement;
                player.ruffle().tick(timestamp as unknown as number);
            },
            this.player,
            timestamp,
        );

        if (this.imageComparisons.has(this.iteration)) {
            await this.captureImage(this.iteration);
        }

        if (this.iteration === this.numTicks) {
            await this.captureImage("last_frame");
        }

        this.iteration += 1;
    }

    async getActualOutput() {
        return (await getAllTraceOutput(this.browser!, this.player))
            .map((line) => line + "\n")
            .join("");
    }

    async assertOutput() {
        expect(await this.getActualOutput()).toEqual(this.outputTxt);
    }

    async assertImage(trigger: Trigger) {
        const comparison: ImageComparison = this.imageComparisons.get(trigger)!;

        const expectedImagePath = path.join(this.testDir, comparison.filename);
        const expectedImage = await readPng(expectedImagePath);
        const actualImage = this.images.get(trigger)!;

        expect([actualImage.width, actualImage.height]).toEqual([
            expectedImage.width,
            expectedImage.height,
        ]);
        // just to make sure
        expect(actualImage.data.length).toEqual(expectedImage.data.length);

        // TODO checks
        expect(actualImage.data).toEqual(expectedImage.data);
    }
}

async function loadTestsDescriptor(): Promise<any> {
    return json5.parse(await fs.readFile(TESTS_DESCRIPTOR, "utf8"));
}

async function loadTest(name: string, testDescriptor: TestDescriptor) {
    const test = new SwfTest(name, testDescriptor);
    await test.configure();

    describe(name, () => {
        it("load test", async function () {
            const quality = {
                1: "low",
                2: "medium",
                4: "high",
                8: "8x8",
                16: "16x16",
            }[test.sampleCount ?? 1]!;

            const swfUrl = `http://localhost:4567/swf_tests/${name}/test.swf`;
            await openTest(
                browser,
                "swf_tests",
                `index.html?swf=${encodeURI(swfUrl)}` +
                    `&config_quality=${encodeURI(quality)}` +
                    `&name=${encodeURI(name)}`,
            );
            await waitForRuffle(browser);
            await throwIfError(browser);

            const player = await browser.$("<ruffle-player>");
            await waitForPlayerToLoad(browser, player);
            await setupAndPlay(browser, player);
            await hideHardwareAccelerationModal(browser, player);

            await test.load(browser, player);
        });

        it("run test", async function () {
            if (!test.loadSucceeded) {
                this.skip();
            }

            let timestamp = 0;
            await test.tick(timestamp);

            for (let i = 0; i < test.numTicks; i++) {
                timestamp += test.frameTime;
                await test.runInputForTick();
                await test.tick(timestamp);
            }
        });

        it("verify output.txt", async function () {
            if (!test.loadSucceeded) {
                this.skip();
            }

            await test.assertOutput();
        });

        for (const [trigger, comparison] of test.imageComparisons) {
            it(`verify ${comparison.filename}`, async function () {
                if (!test.loadSucceeded) {
                    this.skip();
                }

                await test.assertImage(trigger);
            });
        }
    });
}

const testsDescriptor = await loadTestsDescriptor();
for (const key in testsDescriptor["tests"]) {
    await loadTest(key, testsDescriptor.tests[key] as TestDescriptor);
}

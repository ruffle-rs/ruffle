const path = require("path");

async function isRuffleLoaded(browser) {
    return await browser.execute(
        () =>
            window !== undefined &&
            window.RufflePlayer !== undefined &&
            window.RufflePlayer.invoked,
    );
}

async function waitForRuffle(browser) {
    await browser.waitUntil(async () => await isRuffleLoaded(browser), {
        timeoutMsg: "Expected Ruffle to load",
    });
    await throwIfError(browser);
}

async function setupErrorHandler(browser) {
    await browser.execute(() => {
        window.ruffleErrors = [];
        window.addEventListener("error", (error) => {
            window.ruffleErrors.push(error);
        });
    });
}

async function hasError(browser) {
    return await browser.execute(
        () => window.ruffleErrors && window.ruffleErrors.length > 0,
    );
}

async function throwIfError(browser) {
    return await browser.execute(() => {
        if (window.ruffleErrors && window.ruffleErrors.length > 0) {
            throw window.ruffleErrors[0];
        }
    });
}

async function injectRuffle(browser) {
    await setupErrorHandler(browser);
    await browser.execute(() => {
        const script = document.createElement("script");
        script.type = "text/javascript";
        script.src = "/dist/ruffle.js";
        document.head.appendChild(script);
    });
    await throwIfError(browser);
}

async function playAndMonitor(browser, player, expectedOutput) {
    await throwIfError(browser);

    // TODO: better way to test for this in the API
    await browser.waitUntil(
        async () =>
            (await hasError(browser)) ||
            (await browser.execute((player) => player.instance, player)),
        {
            timeoutMsg: "Expected player to have initialized",
        },
    );

    await browser.execute((player) => {
        player.__ruffle_log__ = "";
        player.traceObserver = (msg) => {
            player.__ruffle_log__ += msg + "\n";
        };
        player.play();
    }, player);

    if (expectedOutput === undefined) {
        expectedOutput = "Hello from Flash!\n";
    }

    await browser.waitUntil(
        async () =>
            (await browser.execute(
                (player) => player.__ruffle_log__,
                player,
            )) === expectedOutput,
        {
            timeoutMsg: "Expected Ruffle to trace a message",
        },
    );
}

async function injectRuffleAndWait(browser) {
    await injectRuffle(browser);
    await waitForRuffle(browser);
}

async function openTest(browser, absoluteDir, filename) {
    const dirname = path.basename(absoluteDir);
    if (filename === undefined) {
        filename = "index.html";
    }
    await browser.url(
        `http://localhost:4567/test/polyfill/${dirname}/${filename}`,
    );
}

/** Test set-up for JS API testing. */
function jsApiBefore(swf) {
    let player = null;

    before("Loads the test", async () => {
        await browser.url("http://localhost:4567/test_assets/js_api.html");

        await injectRuffleAndWait(browser);

        player = await browser.execute(() => {
            const ruffle = window.RufflePlayer.newest();
            const player = ruffle.createPlayer();
            const container = document.getElementById("test-container");
            container.appendChild(player);
            return player;
        });

        if (swf) {
            await browser.execute((player) => {
                player.load("/test_assets/example.swf");
            }, player);
            await playAndMonitor(browser, player);
        }
    });
}

module.exports = {
    isRuffleLoaded,
    waitForRuffle,
    playAndMonitor,
    injectRuffle,
    injectRuffleAndWait,
    openTest,
    setupErrorHandler,
    jsApiBefore,
};

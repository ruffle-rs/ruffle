const path = require("path");

function is_ruffle_loaded(browser) {
    return browser.execute(
        () =>
            window !== undefined &&
            window.RufflePlayer !== undefined &&
            window.RufflePlayer.invoked
    );
}

function wait_for_ruffle(browser) {
    browser.waitUntil(() => is_ruffle_loaded(browser), {
        timeoutMsg: "Expected Ruffle to load",
    });
    throw_if_error(browser);
}

function setup_error_handler(browser) {
    browser.execute(() => {
        window.ruffleErrors = [];
        window.addEventListener("error", (error) => {
            window.ruffleErrors.push(error);
        });
    });
}

function has_error(browser) {
    return browser.execute(
        () => window.ruffleErrors && window.ruffleErrors.length > 0
    );
}

function throw_if_error(browser) {
    return browser.execute(() => {
        if (window.ruffleErrors && window.ruffleErrors.length > 0) {
            throw window.ruffleErrors[0];
        }
    });
}

function inject_ruffle(browser) {
    setup_error_handler(browser);
    browser.execute(() => {
        const script = document.createElement("script");
        script.type = "text/javascript";
        script.src = "/dist/ruffle.js";
        document.head.appendChild(script);
    });
    throw_if_error(browser);
}

function play_and_monitor(browser, player, expected_output) {
    throw_if_error(browser);

    // TODO: better way to test for this in the API
    browser.waitUntil(
        () =>
            has_error(browser) ||
            browser.execute((player) => player.instance, player),
        {
            timeoutMsg: "Expected player to have initialized",
        }
    );

    browser.execute((player) => {
        player.__ruffle_log__ = "";
        player.traceObserver = (msg) => {
            player.__ruffle_log__ += msg + "\n";
        };
        player.play();
    }, player);

    if (expected_output === undefined) {
        expected_output = "Hello from Flash!\n";
    }

    browser.waitUntil(
        () =>
            browser.execute((player) => player.__ruffle_log__, player) ===
            expected_output,
        {
            timeoutMsg: "Expected Ruffle to trace a message",
        }
    );
}

function inject_ruffle_and_wait(browser) {
    inject_ruffle(browser);
    wait_for_ruffle(browser);
}

function open_test(browser, absolute_dir, file_name) {
    const dir_name = path.basename(absolute_dir);
    if (file_name === undefined) {
        file_name = "index.html";
    }
    browser.url(`http://localhost:4567/test/polyfill/${dir_name}/${file_name}`);
}

/** Test set-up for JS API testing. */
function js_api_before(swf) {
    let player = null;

    before("Loads the test", () => {
        browser.url("http://localhost:4567/test_assets/js_api.html");

        inject_ruffle_and_wait(browser);

        player = browser.execute(() => {
            const ruffle = window.RufflePlayer.newest();
            const player = ruffle.createPlayer();
            const container = document.getElementById("test-container");
            container.appendChild(player);
            return player;
        });

        if (swf) {
            browser.execute((player) => {
                player.load("/test_assets/example.swf");
            }, player);
            play_and_monitor(browser, player);
        }
    });
}

module.exports = {
    is_ruffle_loaded,
    wait_for_ruffle,
    play_and_monitor,
    inject_ruffle,
    inject_ruffle_and_wait,
    open_test,
    setup_error_handler,
    js_api_before,
};

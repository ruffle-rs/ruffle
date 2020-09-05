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
}

function inject_ruffle(browser) {
    browser.execute(() => {
        const script = document.createElement("script");
        script.type = "text/javascript";
        script.src = "/dist/ruffle.js";
        document.head.appendChild(script);
    });
}

function play_and_monitor(browser, player) {
    // TODO: better way to test for this in the API
    browser.waitUntil(
        () => {
            return browser.execute((player) => {
                return (
                    player.play_button_clicked !== undefined && player.instance
                );
            }, player);
        },
        {
            timeoutMsg: "Expected player to have initialized",
        }
    );

    browser.execute((player) => {
        player.__ruffle_log__ = "";
        player.trace_observer = (msg) => {
            player.__ruffle_log__ += msg + "\n";
        };

        // TODO: make this an actual intended api...
        player.play_button_clicked();
    }, player);

    browser.waitUntil(
        () => {
            return (
                browser.execute((player) => {
                    return player.__ruffle_log__;
                }, player) === "Hello from Flash!\n"
            );
        },
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

module.exports = {
    is_ruffle_loaded,
    wait_for_ruffle,
    play_and_monitor,
    inject_ruffle,
    inject_ruffle_and_wait,
    open_test,
};

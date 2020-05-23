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
        timeout: 5000,
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

function inject_ruffle_and_wait(browser) {
    inject_ruffle(browser);
    wait_for_ruffle(browser);

    console.log(browser.$("html").getHTML(true));
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
    inject_ruffle,
    inject_ruffle_and_wait,
    open_test,
};

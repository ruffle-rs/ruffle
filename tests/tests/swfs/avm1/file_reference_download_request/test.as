// The order of the events and their arguments match Flash Player 32 for
// the same downloads from a real server, where the progress is reported
// for every chunk of the response as it arrives. The requests are logged
// by the test runner.

var downloads = [
    { url: "http://localhost:8000/download.txt?debug-chunk-size=16", postData: undefined },
    // Without a Content-Length, Flash Player reports the total as -1.
    { url: "http://localhost:8000/download.txt?debug-chunk-size=16&debug-no-content-length", postData: undefined },
    // An empty response is an error.
    { url: "http://localhost:8000/empty.txt", postData: undefined },
    { url: "http://localhost:8000/download.txt?debug-chunk-size=16", postData: "x=1&y=two%20words" }
];
var current = -1;
var waitingForClick = false;

var file = new flash.net.FileReference();
var listener = {};

function next() {
    current++;
    if (current < downloads.length) {
        trace("// download " + current + (downloads[current].postData === undefined ? "" : " with postData"));
        if (downloads[current].postData !== undefined) {
            file.postData = downloads[current].postData;
        }
        download();
    } else {
        trace("// done");
    }
}

function download() {
    // Flash Player only allows downloading in response to a user action, so
    // there, every download starts with a click.
    var result = file.download(downloads[current].url, "debug-success.txt");
    trace("download() returned " + result);
    waitingForClick = !result;
}

_root.onMouseDown = function() {
    if (waitingForClick) {
        download();
    }
};

listener.onSelect = function(f) {
    trace("onSelect(" + f.name + ")");
};
listener.onCancel = function(f) {
    trace("onCancel(" + f.name + ")");
};
listener.onOpen = function(f) {
    trace("onOpen(" + f.name + ")");
};
listener.onProgress = function(f, loaded, total) {
    trace("onProgress(" + f.name + ", " + loaded + ", " + total + ")");
};
listener.onComplete = function(f) {
    trace("onComplete(" + f.name + ")");
    next();
};
listener.onHTTPError = function(f, status) {
    trace("onHTTPError(" + f.name + ", " + status + ")");
    next();
};
listener.onIOError = function(f) {
    trace("onIOError(" + f.name + ")");
    next();
};
file.addListener(listener);

next();

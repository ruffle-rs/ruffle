// The events and their arguments match Flash Player 32 for the same
// uploads to a real server. The requests are logged by the test runner;
// Flash Player sends the same multipart body, apart from its random
// boundary string. See README.md for how to compare them.

var uploads = [
    { url: "http://localhost:8000/upload?debug-success&q=1", field: "MyField", postData: "a=1&b=two%20words" },
    { url: "http://localhost:8000/upload?debug-success-empty", field: undefined, postData: undefined },
    { url: "http://localhost:8000/upload?debug-error-statuscode=404", field: undefined, postData: undefined },
    { url: "http://localhost:8000/upload?debug-error-statuscode=500", field: "Other", postData: "x=1" },
    // Quotes and line breaks in the field name and postData names are
    // percent-encoded in the multipart headers.
    { url: "http://localhost:8000/upload?debug-success-empty&names", field: "my\"field", postData: "a%22b=1&c%0D%0Ad=2" }
];
var current = -1;
var waitingForClick = false;

var file = new flash.net.FileReference();
var listener = {};

function next() {
    current++;
    if (current < uploads.length) {
        trace("// upload " + current + ": " + uploads[current].url);
        browse();
    } else {
        trace("// done");
    }
}

function browse() {
    // Flash Player only allows browsing in response to a user action, so
    // there, every upload starts with a click.
    waitingForClick = !file.browse([{ description: "debug-select-success", extension: "*.txt" }]);
}

_root.onMouseDown = function() {
    if (waitingForClick) {
        browse();
    }
};

listener.onSelect = function(f) {
    trace("onSelect(" + f.name + ")");
    var upload = uploads[current];
    if (upload.postData !== undefined) {
        f.postData = upload.postData;
    }
    var result = upload.field === undefined ? f.upload(upload.url) : f.upload(upload.url, upload.field);
    trace("upload() returned " + result);
};
listener.onOpen = function(f) {
    trace("onOpen(" + f.name + ")");
};
listener.onComplete = function(f) {
    trace("onComplete(" + f.name + ")");
    if (uploads[current].url.indexOf("debug-success-empty") != -1) {
        next();
    }
};
listener.onUploadCompleteData = function(f, data) {
    trace("onUploadCompleteData(" + f.name + ", " + data + ")");
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

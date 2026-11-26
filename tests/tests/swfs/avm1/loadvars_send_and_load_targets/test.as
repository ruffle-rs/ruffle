// Calls LoadVars.sendAndLoad with different targets, URLs and methods and
// traces what it returns, when the arguments and `this` are converted to
// strings and which targets get loaded.

function watch(name, target) {
    target.onData = function(src) {
        trace(name + ": onData(" + src + ")");
    };
    return target;
}

var source = new LoadVars();
source.value = "sent";

var targets = [
    ["LoadVars", watch("LoadVars", new LoadVars())],
    ["XML", watch("XML", new XML())],
    ["Object", watch("Object", {})],
    ["Array", watch("Array", [])],
    ["clip", watch("clip", _root.createEmptyMovieClip("clip", 1))],
    ["TextField", watch("TextField", _root.createTextField("field", 2, 0, 0, 100, 20))],
    ["string", "text"],
    ["number", 5],
    ["undefined", undefined],
    ["null", null]
];

for (var i = 0; i < targets.length; i++) {
    trace(targets[i][0] + ": returned " + source.sendAndLoad("data.txt", targets[i][1]));
}

trace("no target: returned " + source.sendAndLoad("data.txt"));
trace("no arguments: returned " + source.sendAndLoad());
trace("null URL: returned " + source.sendAndLoad(null, watch("null URL", new LoadVars())));
trace("undefined URL: returned " + source.sendAndLoad(undefined, watch("undefined URL", new LoadVars())));
trace("GET: returned " + source.sendAndLoad("data.txt", watch("GET", new LoadVars()), "GET"));

var xml = new XML("<request/>");
xml.sendAndLoad = LoadVars.prototype.sendAndLoad;
trace("XML as this: returned " + xml.sendAndLoad("data.txt", watch("XML as this", new LoadVars())));

var plain = {sendAndLoad: LoadVars.prototype.sendAndLoad};
trace("Object as this: returned " + plain.sendAndLoad("data.txt", watch("Object as this", new LoadVars())));

// Shows when the URL and the method are converted to strings.
var loggingUrl = {toString: function() {
    trace("URL converted");
    return "data.txt";
}};
var loggingMethod = {toString: function() {
    trace("method converted");
    return "POST";
}};

trace("logging URL, LoadVars target: returned " + source.sendAndLoad(loggingUrl, watch("logging URL", new LoadVars()), loggingMethod));
trace("logging URL, number target: returned " + source.sendAndLoad(loggingUrl, 5, loggingMethod));
trace("logging URL, clip target: returned " + source.sendAndLoad(loggingUrl, _root.clip, loggingMethod));

// Shows whether `this` is converted to a string and whether its properties
// are read directly.
var loggingThis = new LoadVars();
loggingThis.addProperty("value", function() {
    trace("value read");
    return "sent";
}, null);
loggingThis.toString = function() {
    trace("this converted");
    return "value=logging";
};

trace("logging this, LoadVars target: returned " + loggingThis.sendAndLoad(loggingUrl, watch("logging this", new LoadVars()), loggingMethod));
trace("logging this, number target: returned " + loggingThis.sendAndLoad(loggingUrl, 5, loggingMethod));

var frames = 0;
_root.onEnterFrame = function() {
    frames++;
    if (frames == 8) {
        trace("Done");
        delete _root.onEnterFrame;
    }
};

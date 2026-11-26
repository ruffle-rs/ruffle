// Calls XML.sendAndLoad with different targets and traces what it returns
// and which targets get loaded.

// Display objects only get an onData handler, so that the output doesn't
// depend on their own load events.
function watchData(name, target) {
    target.onData = function(src) {
        trace(name + ": onData(" + src + ")");
    };
    return target;
}

function watch(name, target) {
    target.onLoad = function(success) {
        trace(name + ": onLoad(" + success + "), loaded = " + this.loaded);
    };
    target.onData = function(src) {
        trace(name + ": onData(" + src + ")");
        if (this instanceof XML) {
            this.parseXML(src);
            this.loaded = true;
            this.onLoad(true);
        }
    };
    return target;
}

var source = new XML("<request/>");

var targets = [
    ["XML", watch("XML", new XML())],
    ["clip", watchData("clip", _root.createEmptyMovieClip("clip", 1))],
    ["TextField", watchData("TextField", _root.createTextField("field", 2, 0, 0, 100, 20))],
    ["Object", watch("Object", {})],
    ["LoadVars", watch("LoadVars", new LoadVars())],
    ["Array", watch("Array", [])],
    ["Function", watch("Function", function() {})],
    ["string", "text"],
    ["number", 5],
    ["undefined", undefined],
    ["null", null]
];

for (var i = 0; i < targets.length; i++) {
    trace(targets[i][0] + ": returned " + source.sendAndLoad("data.xml", targets[i][1]));
}

trace("no target: returned " + source.sendAndLoad("data.xml"));
trace("no arguments: returned " + source.sendAndLoad());
trace("null URL: returned " + source.sendAndLoad(null, watch("null URL", new XML())));
trace("undefined URL: returned " + source.sendAndLoad(undefined, watch("undefined URL", new XML())));

var vars = new LoadVars();
vars.sendAndLoad = XML.prototype.sendAndLoad;
trace("LoadVars as this: returned " + vars.sendAndLoad("data.xml", watch("LoadVars as this", new XML())));

var plain = {sendAndLoad: XML.prototype.sendAndLoad};
trace("Object as this: returned " + plain.sendAndLoad("data.xml", watch("Object as this", new XML())));

// Shows when the URL and `this` are converted to strings.
var loggingUrl = {toString: function() {
    trace("URL converted");
    return "data.xml";
}};
var loggingThis = {
    sendAndLoad: XML.prototype.sendAndLoad,
    toString: function() {
        trace("this converted");
        return "logging";
    }
};

trace("logging URL, XML target: returned " + source.sendAndLoad(loggingUrl, watch("logging URL", new XML())));
trace("logging URL, number target: returned " + source.sendAndLoad(loggingUrl, 5));
trace("logging this, XML target: returned " + loggingThis.sendAndLoad(loggingUrl, watch("logging this", new XML())));
trace("logging this, number target: returned " + loggingThis.sendAndLoad(loggingUrl, 5));

// A real XML object whose toString is overridden.
var loggingXml = new XML("<request/>");
loggingXml.toString = function() {
    trace("XML converted");
    return "<logging/>";
};

trace("logging XML, XML target: returned " + loggingXml.sendAndLoad(loggingUrl, watch("logging XML", new XML())));
trace("logging XML, number target: returned " + loggingXml.sendAndLoad(loggingUrl, 5));

// Shows which objects get the load properties.
function showLoadProperties(name, object) {
    trace(name + ": loaded " + object.hasOwnProperty("loaded") + " " + object.loaded
        + ", _bytesLoaded " + object.hasOwnProperty("_bytesLoaded") + " " + object._bytesLoaded
        + ", _bytesTotal " + object.hasOwnProperty("_bytesTotal") + " " + object._bytesTotal);
}

var sender = new XML("<request/>");
var target = {};
sender.sendAndLoad("data.xml", target);
showLoadProperties("sender", sender);
showLoadProperties("target", target);

var frames = 0;
_root.onEnterFrame = function() {
    frames++;
    if (frames == 8) {
        trace("Done");
        delete _root.onEnterFrame;
    }
};

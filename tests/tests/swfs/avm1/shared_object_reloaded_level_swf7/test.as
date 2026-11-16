// SharedObject.getLocal called by bytecode of the root movie and of a child
// movie, on levels that held the child and then loaded another movie, or
// failed to. Each label names the movie whose bytecode calls getLocal.

var reloadUrls = [
    "foo:/nonexistent/a.swf",
    "foo:nonexistent/a.swf",
    "foo:///nonexistent/a.swf",
    "foo://host/nonexistent/a.swf",
    "nonexistent/a.swf",
    "other/child.swf"
];
var localPaths = [null, "/", "/nonexistent", "/nonexistent/a.swf"];
var objects = [];
var labels = [];
var saved = [];
var loadedCount = 0;
var interval;

function rootGetLocal(name, localPath) {
    return SharedObject.getLocal(name, localPath);
}

function report(so, label) {
    if (so == null) {
        trace(label + ": " + so);
        return;
    }
    for (var i = 0; i < objects.length; i++) {
        if (objects[i] == so) {
            trace(label + ": same as " + labels[i]);
            return;
        }
    }
    objects.push(so);
    labels.push(label);
    so.data.label = label;
    if (typeof so.flush == "function") {
        trace(label + ": new, " + so.flush());
    } else {
        trace(label + ": new, flush is " + typeof so.flush);
    }
}

function checkCall(label, getter) {
    for (var i = 0; i < localPaths.length; i++) {
        report(getter("so", localPaths[i]), label + ", " + localPaths[i]);
    }
}

function checkMethod(label, level, getter) {
    level.getter = getter;
    for (var i = 0; i < localPaths.length; i++) {
        report(level.getter("so", localPaths[i]), label + ", " + localPaths[i]);
    }
}

// Called by each child when its frame runs.
function childLoaded() {
    loadedCount++;
    if (loadedCount != reloadUrls.length) {
        return;
    }
    trace("// Before the levels load another movie");
    checkCall("root", rootGetLocal);
    checkCall("child", _level1.childGetLocal);
    for (var i = 0; i < reloadUrls.length; i++) {
        saved[i] = eval("_level" + (i + 1)).childGetLocal;
        loadMovieNum(reloadUrls[i], i + 1);
    }
    interval = setInterval(afterReload, 500);
}

function afterReload() {
    clearInterval(interval);
    trace("// After the levels loaded another movie, or failed to");
    for (var i = 0; i < reloadUrls.length; i++) {
        var name = "_level" + (i + 1);
        var level = eval(name);
        var url = level._url;
        if (url.indexOf("file:") == 0) {
            // Leave out the directory of the test.
            url = "file:..." + url.substr(url.lastIndexOf("/", url.lastIndexOf("/") - 1));
        }
        trace("// " + name + ": " + reloadUrls[i] + ", " + typeof level + ", " + url);
        checkCall(name + ", child, called by root", saved[i]);
        checkMethod(name + ", child, method of the level", level, saved[i]);
        checkMethod(name + ", root, method of the level", level, rootGetLocal);
    }
}

for (var i = 0; i < reloadUrls.length; i++) {
    loadMovieNum("sub/child.swf", i + 1);
}

// Loaded over HTTP into _level10 by the local root. Reports shared objects
// of this movie and of movies from the same domain: one loaded into _level11
// by this movie, and one loaded into _level12 by the local root.

var localPaths = [null, "/"];
var objects = [];
var labels = [];
var urls = ["http://localhost:8000/a/child.swf", "http://localhost:8000/d/child.swf"];
var names = ["child a on _level11, loaded by host", "child d on _level12, loaded by root"];
var loaded = {};
var index = 0;

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

// Called by each child when its frame runs. The children are reported in
// the order of urls, whatever order they load in.
function childLoaded(child) {
    loaded[child._url] = child;
    while (index < urls.length && loaded[urls[index]] != undefined) {
        child = loaded[urls[index]];
        var name = names[index];
        report(child.calledByChild, name + ", frame, null");
        for (var i = 0; i < localPaths.length; i++) {
            report(child.childGetLocal("so", localPaths[i]), name + ", called by host, " + localPaths[i]);
        }
        index++;
    }
}

trace("// Movies from the same domain over HTTP");
for (var i = 0; i < localPaths.length; i++) {
    report(SharedObject.getLocal("so", localPaths[i]), "host, " + localPaths[i]);
}
loadMovieNum(urls[0], 11);

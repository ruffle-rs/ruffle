// SharedObject.getLocal called by bytecode of the root movie or of a child
// movie on _level1, where the other movie is the caller, `this`, the target,
// in the scope chain, or the level of a callback. Then a movie loaded over
// HTTP into _level10 reports the same for movies from its domain.
// Each label names the movie whose bytecode calls getLocal.

var localPaths = [null, "/", "/nonexistent"];
var objects = [];
var labels = [];
var definedInWith;
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

function check(label, getter) {
    for (var i = 0; i < localPaths.length; i++) {
        report(getter(localPaths[i]), label + ", " + localPaths[i]);
    }
}

// Called by the child when its frame runs.
function childLoaded() {
    trace("// Bytecode called by its own movie");
    check("root", function(localPath) {
        return rootGetLocal("so", localPath);
    });
    report(_level1.calledByChild, "child frame, null");

    trace("// Bytecode called by the other movie");
    check("child, called by root", function(localPath) {
        return _level1.childGetLocal("so", localPath);
    });
    check("root, called by child", function(localPath) {
        return _level1.callGetLocal(rootGetLocal, "so", localPath);
    });

    trace("// Bytecode with the other movie's level as this");
    _level1.rootMethod = rootGetLocal;
    check("root, method of child", function(localPath) {
        return _level1.rootMethod("so", localPath);
    });
    _level0.childMethod = _level1.childGetLocal;
    check("child, method of root", function(localPath) {
        return _level0.childMethod("so", localPath);
    });
    if (typeof rootGetLocal.call == "function") {
        check("root, called with child as this", function(localPath) {
            return rootGetLocal.call(_level1, "so", localPath);
        });
    } else {
        trace("root, called with child as this: call is " + typeof rootGetLocal.call);
    }

    trace("// Root bytecode with the child in the scope chain or as target");
    check("root, in with (child)", function(localPath) {
        with (_level1) {
            return SharedObject.getLocal("so", localPath);
        }
    });
    with (_level1) {
        definedInWith = function(localPath) {
            return SharedObject.getLocal("so", localPath);
        };
    }
    check("root, defined in with (child)", definedInWith);
    check("root, with child as target", function(localPath) {
        var so;
        tellTarget ("_level1") {
            so = SharedObject.getLocal("so", localPath);
        }
        return so;
    });

    _level1.onEnterFrame = function() {
        delete this.onEnterFrame;
        trace("// Root bytecode called back on the child's level");
        report(SharedObject.getLocal("so", null), "root, onEnterFrame of child, null");
    };
    interval = setInterval(loadHttp, 300);
}

function loadHttp() {
    clearInterval(interval);
    loadMovieNum("http://localhost:8000/host.swf", 10);
    interval = setInterval(loadHttpChild, 500);
}

function loadHttpChild() {
    clearInterval(interval);
    loadMovieNum("http://localhost:8000/d/child.swf", 12);
}

loadMovieNum("sub/child.swf", 1);

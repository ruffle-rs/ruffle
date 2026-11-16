stop();

var host = this;
var cases = [];
var frame = 0;
var directReadyFrame = -1;
var effects = 0;
var replacementUrl = host._url.substring(
    0, host._url.lastIndexOf("/") + 1
) + "replacement.swf";

host.replacementRuns = 0;
var directLoad = host.createEmptyMovieClip("direct_load", 7);


function makeCase(label, operation, depth) {
    var clip = host.createEmptyMovieClip(label, depth);
    var record = {
        label: label,
        operation: operation,
        clip: clip,
        unloadCalls: 0,
        enterCalls: 0,
        enterSnapshot: 0
    };

    clip.customValue = 123;
    if (label == "unload_once") {
        record.watchCalls = 0;
        clip.watched = 10;
        clip.watch("watched", function(name, oldValue, newValue, userData) {
            record.watchCalls = record.watchCalls + 1;
            return newValue;
        });
    }


    clip.onUnload = function() {
        record.unloadCalls = record.unloadCalls + 1;
        record.handlerCustomType = typeof(this.customValue);
        record.handlerDefaultPrototype = (this.__proto__ === MovieClip.prototype);
        this.fromUnload = 789;

        if (record.label == "unload_in_handler" && !record.didReenter) {
            record.didReenter = true;
            this.unloadMovie();
        }

        if (record.label == "load_in_handler" && !record.didReload) {
            record.didReload = true;
            this.loadMovie(replacementUrl);
        }

        effects++;
        host.createEmptyMovieClip("effect" + effects, 100 + effects);
    };

    clip.onEnterFrame = function() {
        record.enterCalls = record.enterCalls + 1;
    };

    var customPrototype = { inheritedValue: 456 };
    customPrototype.__proto__ = clip.__proto__;
    clip.__proto__ = customPrototype;
    record.savedPrototype = customPrototype;

    return record;
}

cases.push(makeCase("unload_once", "unload", 1));
cases.push(makeCase("unload_repeated", "unload", 2));
cases.push(makeCase("remove_once", "remove", 3));
cases.push(makeCase("unload_protected", "unload", 4));
ASSetPropFlags(
    cases[3].clip,
    "customValue,onUnload,onEnterFrame",
    7,
    0
);


cases.push(makeCase("unload_in_handler", "unload", 5));
var reloadCase = makeCase("load_in_handler", "unload", 6);
cases.push(reloadCase);

host.onEnterFrame = function() {
    frame++;
    if (directReadyFrame < 0 && host.direct_load.replacementMarker == "loaded") {
        directReadyFrame = frame;
    }


    if (frame == 3) {
        trace("before unload");
        directLoad.loadMovie(replacementUrl);

        for (var i = 0; i < cases.length; i++) {
            var c = cases[i];
            trace(c.label + " entered=" + (c.enterCalls > 0));

            if (c.operation == "remove") {
                c.clip.removeMovieClip();
            } else {
                c.clip.unloadMovie();
            }

            trace(c.label + " immediateCustom=" + typeof(c.clip.customValue));
        }
    }

    if (frame == 5) {
        trace("properties after unload");

        for (var i = 0; i < cases.length; i++) {
            var c = cases[i];
            c.enterSnapshot = c.enterCalls;

            // Inspect the replacement after allowing its load to finish.
            if (c.label == "load_in_handler") {
                continue;
            }


            trace(c.label
                + " custom=" + typeof(c.clip.customValue)
                + " onUnload=" + typeof(c.clip.onUnload)
                + " onEnterFrame=" + typeof(c.clip.onEnterFrame));
            trace(c.label
                + " samePrototype=" + (c.clip.__proto__ === c.savedPrototype)
                + " inherited=" + typeof(c.clip.inheritedValue));
            trace(c.label
                + " prototypeType=" + typeof(c.clip.__proto__)
                + " defaultPrototype=" + (c.clip.__proto__ === MovieClip.prototype)
                + " nameType=" + typeof(c.clip._name));
            trace(c.label + " fromUnload=" + typeof(c.clip.fromUnload));
            if (c.label == "unload_once") {
                c.clip.watched = 42;
            }

        }
    }

    if (frame == 7 || frame == 9 || frame == 11) {
        cases[1].clip.unloadMovie();
    }

    if (frame >= 15 && (
        (directReadyFrame >= 0 && frame >= directReadyFrame + 12)
        || frame >= 120
    )) {
        trace("final callbacks");

        for (var i = 0; i < cases.length; i++) {
            var c = cases[i];

            trace("handler " + c.label
                + " custom=" + c.handlerCustomType
                + " defaultPrototype=" + c.handlerDefaultPrototype);

            trace(c.label
                + " unloadCalls=" + c.unloadCalls
                + " enterContinued=" + (c.enterCalls > c.enterSnapshot));
        }

        trace("watcher calls=" + cases[0].watchCalls
            + " value=" + cases[0].clip.watched
            + " unwatch=" + cases[0].clip.unwatch("watched"));
        trace("replacement marker=" + host.load_in_handler.replacementMarker);
        trace("direct marker=" + host.direct_load.replacementMarker);
        trace("replacement runs=" + host.replacementRuns);
        trace("effects=" + effects);
        trace("UNLOAD_CALLBACKS_COMPLETE");
        delete host.onEnterFrame;
    }
};

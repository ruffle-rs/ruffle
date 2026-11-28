stop();

var host = this;
var records = [];
var frame = 0;

function observe(label, clip) {
    var record = {
        label: label,
        clip: clip,
        unloadCalls: 0,
        watchCalls: 0
    };

    clip.customValue = 123;
    clip.watched = 10;

    var prototype = { inheritedValue: 456 };
    prototype.__proto__ = clip.__proto__;
    clip.__proto__ = prototype;

    clip.watch("watched", function(name, oldValue, newValue, userData) {
        record.watchCalls = record.watchCalls + 1;
        return newValue;
    });

    clip.onUnload = function() {
        record.unloadCalls = record.unloadCalls + 1;
        record.handlerCustomType = typeof(this.customValue);
        record.handlerThis = this;
    };

    records.push(record);
}

var unloadParent = createEmptyMovieClip("unloadParent", 1);
var unloadChild = unloadParent.createEmptyMovieClip("child", 1);
var removeParent = createEmptyMovieClip("removeParent", 2);
var removeChild = removeParent.createEmptyMovieClip("child", 1);

observe("unload parent", unloadParent);
observe("unload child", unloadChild);
observe("remove parent", removeParent);
observe("remove child", removeChild);

host.onEnterFrame = function() {
    frame++;

    if (frame == 3) {
        unloadParent.unloadMovie();
        removeParent.removeMovieClip();
    }

    if (frame == 6) {
        trace("after unload");

        for (var i = 0; i < records.length; i++) {
            var r = records[i];

            trace(r.label
                + " custom=" + typeof(r.clip.customValue)
                + " onUnload=" + typeof(r.clip.onUnload)
                + " inherited=" + typeof(r.clip.inheritedValue)
                + " defaultPrototype=" + (r.clip.__proto__ === MovieClip.prototype)
                + " nameType=" + typeof(r.clip._name));

            trace(r.label
                + " handlerThis.custom=" + typeof(r.handlerThis.customValue)
                + " handlerThis.name=" + typeof(r.handlerThis._name));

            r.clip.watched = 42;
        }
    }

    if (frame == 9) {
        trace("final callbacks");

        for (var i = 0; i < records.length; i++) {
            var r = records[i];

            trace(r.label
                + " unloadCalls=" + r.unloadCalls
                + " handlerCustom=" + r.handlerCustomType
                + " watchCalls=" + r.watchCalls
                + " watched=" + r.clip.watched);
        }

        trace("parent unwatch=" + unloadParent.unwatch("watched"));
        trace("NESTED_UNLOAD_COMPLETE");
        delete host.onEnterFrame;
    }
};

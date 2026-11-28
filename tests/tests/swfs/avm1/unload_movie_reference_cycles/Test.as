stop();

var host = this;
var frame = 0;
var state = {
    phase: "create",
    ready: false,
    events: [],
    keepAlive: []
};

MovieClip.prototype.onConstruct = function() {
    state.keepAlive.push(super);

    var message = "construct " + state.phase
        + " before=" + typeof(this.marker);

    this.marker = "constructed-" + state.phase;
    state.latest = this;

    if (state.ready) {
        message = message
            + " self=" + this.marker
            + " used=" + state.used.marker
            + " idle=" + state.idle.marker
            + " latest=" + state.latest.marker;
    }

    state.events.push(message);
};

function unloadHandler() {
    state.events.push(
        "unload " + state.phase
        + " self=" + this.marker
        + " used=" + state.used.marker
        + " idle=" + state.idle.marker
        + " latest=" + state.latest.marker
    );
}

createEmptyMovieClip("mc", 1);

// Separate property reads create independent saved references.
state.used = host.mc;
state.idle = host.mc;
state.ready = true;

host.mc.marker = "first";
host.mc.onUnload = unloadHandler;

host.onEnterFrame = function() {
    frame++;

    if (frame == 3) {
        state.phase = "first";
        host.mc.unloadMovie();
    }

    if (frame == 8) {
        host.mc.marker = "second";
        host.mc.onUnload = unloadHandler;

        // Only this reference is used between the first two unloads.
        state.events.push("between used=" + state.used.marker);
    }

    if (frame == 11) {
        state.phase = "second";
        host.mc.unloadMovie();
    }

    if (frame == 15) {
        state.events.push(
            "after second used=" + state.used.marker
            + " idle=" + state.idle.marker
            + " latest=" + state.latest.marker
        );

        host.mc._name = "renamed";
        host.renamed.marker = "third";
        host.renamed.onUnload = unloadHandler;
    }

    if (frame == 19) {
        state.phase = "third";
        host.renamed.unloadMovie();
    }

    if (frame == 25) {
        for (var i = 0; i < state.events.length; i++) {
            trace(state.events[i]);
        }

        trace("final used=" + state.used.marker
            + " idle=" + state.idle.marker
            + " latest=" + state.latest.marker
            + " viaParent=" + host.renamed.marker);

        trace("retained supers=" + state.keepAlive.length
            + " firstType=" + typeof(state.keepAlive[0]));

        trace("UNLOAD_REFERENCE_CYCLES_COMPLETE");
        delete MovieClip.prototype.onConstruct;
        delete host.onEnterFrame;
    }
};

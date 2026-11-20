stop();

var state = {
    constructCalls: 0,
    unloadCalls: 0,
    events: []
};
var frame = 0;

MovieClip.prototype.onConstruct = function() {
    state.constructCalls = state.constructCalls + 1;

    state.events.push(
        "construct #" + state.constructCalls
        + " custom=" + typeof(this.customValue)
        + " fromConstruct=" + typeof(this.fromConstruct)
        + " x=" + this._x
        + " y=" + this._y
        + " lockroot=" + this._lockroot
    );

    this.fromConstruct = state.constructCalls;
    state.constructedRef = this;
};

var clip = createEmptyMovieClip("mc", 1);

clip.customValue = 123;
clip._x = 37;
clip._y = 49;
clip._lockroot = true;
clip.onRelease = function() {};

clip.onUnload = function() {
    state.unloadCalls = state.unloadCalls + 1;
    state.events.push(
        "unload custom=" + typeof(this.customValue)
        + " fromConstruct=" + this.fromConstruct
        + " viaParentCustom=" + typeof(this._parent.mc.customValue)
        + " viaParentConstruct=" + this._parent.mc.fromConstruct
        + " savedConstruct=" + state.constructedRef.fromConstruct
    );
};

this.onEnterFrame = function() {
    frame++;

    if (frame == 3) {
        trace("before constructCalls=" + state.constructCalls);

        clip.unloadMovie();

        trace("immediate constructCalls=" + state.constructCalls
            + " custom=" + typeof(clip.customValue)
            + " fromConstruct=" + clip.fromConstruct);
    }

    if (frame == 8) {
        for (var i = 0; i < state.events.length; i++) {
            trace(state.events[i]);
        }

        trace("final constructCalls=" + state.constructCalls
            + " unloadCalls=" + state.unloadCalls);

        trace("final custom=" + typeof(clip.customValue)
            + " onRelease=" + typeof(clip.onRelease)
            + " fromConstruct=" + clip.fromConstruct);

        trace("final x=" + clip._x
            + " y=" + clip._y
            + " lockroot=" + clip._lockroot);

        trace("final defaultPrototype="
            + (clip.__proto__ === MovieClip.prototype));

        trace("UNLOAD_CONSTRUCT_COMPLETE");

        delete MovieClip.prototype.onConstruct;
        delete this.onEnterFrame;
    }
};

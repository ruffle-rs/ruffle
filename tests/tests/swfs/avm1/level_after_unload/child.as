var name = String(this);

this.onUnload = function() {
    trace("onUnload: " + this);
};
createEmptyMovieClip("inner", 1);
inner.onUnload = function() {
    trace("onUnload: " + this);
};

_level0.enterFrames[name] = 0;
_level0.intervals[name] = 0;
this.onEnterFrame = function() {
    _level0.enterFrames[name] += 1;
};
setInterval(function() {
    _level0.intervals[name] += 1;
}, 50);

function describeSelf() {
    return "function of " + name;
}

_level0.childLoaded();

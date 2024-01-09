text.onSetFocus = function(oldFocus) {
    trace("==========");
    trace("text.onSetFocus");
    trace(oldFocus);
};
text.onKillFocus = function(newFocus) {
    trace("==========");
    trace("text.onKillFocus");
    trace(newFocus);
};

button.onSetFocus = function(oldFocus) {
    trace("==========");
    trace("button.onSetFocus");
    trace(oldFocus);
};
button.onKillFocus = function(newFocus) {
    trace("==========");
    trace("button.onKillFocus");
    trace(newFocus);
};

_root.createEmptyMovieClip("clip", _root.getNextHighestDepth());
clip.focusEnabled = true;
clip.onSetFocus = function(oldFocus) {
    trace("==========");
    trace("clip.onSetFocus");
    trace(oldFocus);
};
clip.onKillFocus = function(newFocus) {
    trace("==========");
    trace("clip.onKillFocus");
    trace(newFocus);
};

var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    trace("==========");
    trace("Selection.onSetFocus");
    trace(oldFocus);
    trace(newFocus);
};
Selection.addListener(listener);

Selection.setFocus(text);
Selection.setFocus(button);
Selection.setFocus(clip);
Selection.setFocus(null);

trace("==========");
trace("Done");

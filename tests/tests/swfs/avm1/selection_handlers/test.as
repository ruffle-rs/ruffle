text.onSetFocus = function(oldFocus) {
    trace("text.onSetFocus: " + oldFocus);
};
text.onKillFocus = function(newFocus) {
    trace("text.onKillFocus: " + newFocus);
};
text.onRollOver = function() {
    trace("text.onRollOver");
};
text.onRollOut = function() {
    trace("text.onRollOut");
};

button.onSetFocus = function(oldFocus) {
    trace("button.onSetFocus: " + oldFocus);
};
button.onKillFocus = function(newFocus) {
    trace("button.onKillFocus: " + newFocus);
};
button.onRollOver = function() {
    trace("button.onRollOver");
};
button.onRollOut = function() {
    trace("button.onRollOut");
};

_root.createEmptyMovieClip("clip", _root.getNextHighestDepth());
clip.focusEnabled = true;
clip.onSetFocus = function(oldFocus) {
    trace("clip.onSetFocus: " + oldFocus);
};
clip.onKillFocus = function(newFocus) {
    trace("clip.onKillFocus: " + newFocus);
};
clip.onRollOver = function() {
    trace("clip.onRollOver");
};
clip.onRollOut = function() {
    trace("clip.onRollOut");
};

var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    trace("Selection.onSetFocus: " + oldFocus + " -> " + newFocus);
};
Selection.addListener(listener);

trace("Setting text");
Selection.setFocus(text);

trace("Setting text");
Selection.setFocus(text);

trace("Setting button");
Selection.setFocus(button);

trace("Setting button");
Selection.setFocus(button);

trace("Setting clip");
Selection.setFocus(clip);

trace("Setting clip");
Selection.setFocus(clip);

trace("Setting null");
Selection.setFocus(null);

trace("Setting null");
Selection.setFocus(null);

trace("Done");

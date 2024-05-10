_root.focusEnabled = true;
_root.tabEnabled = true;
_root.tabIndex = 1;
_root._focusrect = true;

_root.onRelease = function() {

};

_root.clip.focusEnabled = true;
_root.clip.tabEnabled = true;
_root.clip.tabIndex = 1;
_root.clip._focusrect = true;

var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed: " + oldFocus + " -> " + newFocus);
    }
};
Selection.addListener(listener);

Selection.setFocus(_root);
Selection.setFocus(_root.clip);
trace("=====");

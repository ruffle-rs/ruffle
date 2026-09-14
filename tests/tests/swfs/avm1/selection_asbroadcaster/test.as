trace(Selection._listeners.length);

var focusListener:Object = new Object();
focusListener.onSetFocus = function(oldObj, newObj) {
    trace("focusListener: " + oldObj + "," + newObj);
};
focusListener.onX = function(x) {
    trace("onX: " + x);
};
Selection.addListener(focusListener);

trace(Selection._listeners.length);
trace(Selection._listeners[0]);
Selection._listeners[0].onSetFocus("a", "b");

trace("Broadcast");

Selection.broadcastMessage("onSetFocus", "A", "B");
Selection.broadcastMessage("onX", "X");
Selection.broadcastMessage("onY", "Y");

trace("Remove");

Selection.removeListener(focusListener);
trace(Selection._listeners.length);

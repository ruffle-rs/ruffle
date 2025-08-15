var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    trace("Focus at: " + newFocus);
};
Selection.addListener(listener);

function setHandlers(obj) {
    obj.onPress =          function () { trace(obj + ".onPress"); }
    obj.onRollOut =        function () { trace(obj + ".onRollOut") }
}

clip.focusEnabled = true;
setHandlers(clip);
setHandlers(text);

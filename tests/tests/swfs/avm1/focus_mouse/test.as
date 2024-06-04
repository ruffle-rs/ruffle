var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    trace("Focus at: " + newFocus);
};
Selection.addListener(listener);

function setHandlers(obj) {
    obj.onRelease =        function () { trace(obj + ".onRelease"); }
    obj.onKeyDown =        function () { trace(obj + ".onKeyDown"); }
    obj.onKeyUp =          function () { trace(obj + ".onKeyUp"); }
    obj.onPress =          function () { trace(obj + ".onPress"); }
    obj.onReleaseOutside = function () { trace(obj + ".onReleaseOutside"); }
    obj.onMouseDown =      function () { trace(obj + ".onMouseDown") }
    obj.onMouseUp =        function () { trace(obj + ".onMouseUp") }
}

clip.focusEnabled = true;
setHandlers(clip);
setHandlers(button);
setHandlers(text);

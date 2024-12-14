var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    trace("Focus at: " + newFocus);
};
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        trace("Escape pressed");
        _focusrect = false;
        Selection.setFocus(text);
    }
    if (Key.getCode() == 9) {
        trace("Tab pressed");
    }
};
Selection.addListener(listener);
Key.addListener(listener);

function setHandlers(obj) {
    obj.onRelease =        function () { trace(obj + ".onRelease: " + Key.getCode()); }
    obj.onKeyDown =        function () { trace(obj + ".onKeyDown: " + Key.getCode()); }
    obj.onKeyUp =          function () { trace(obj + ".onKeyUp: " + Key.getCode()); }
    obj.onPress =          function () { trace(obj + ".onPress: " + Key.getCode()); }
    obj.onReleaseOutside = function () { trace(obj + ".onReleaseOutside"); }
    obj.onMouseDown =      function () { trace(obj + ".onMouseDown") }
    obj.onMouseUp =        function () { trace(obj + ".onMouseUp") }
    obj.onMouseMove =      function () { trace(obj + ".onMouseMove") }
}

clip.tabEnabled = true;
clip.tabIndex = 1;
setHandlers(clip);
button.tabEnabled = true;
button.tabIndex = 2;
setHandlers(button);
text.tabEnabled = true;
text.tabIndex = 3;
setHandlers(text);

Selection.setFocus(text);

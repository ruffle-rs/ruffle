var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed: " + oldFocus + " -> " + newFocus);
    }
};
listener.onKeyDown = function() {
    if (Key.getCode() == 9) {
        trace("Tab pressed");
    }
    if (Key.getCode() == 27) {
        trace("Escape pressed");
    }
};
Selection.addListener(listener);
Key.addListener(listener);

button.tabEnabled = true;
button.tabIndex = 1;

button.onMouseDown = function () { trace("button.onMouseDown"); }
button.onMouseMove = function () { trace("button.onMouseMove"); }
button.onMouseUp =   function () { trace("button.onMouseUp"); }
button.onKillFocus = function () { trace("button.onKillFocus"); }
button.onSetFocus =  function () { trace("button.onSetFocus"); }
button.onRollOut =   function () { trace("button.onRollOut"); }
button.onRollOver =  function () { trace("button.onRollOver"); }

button2.tabEnabled = true;
button2.tabIndex = 1;

button2.onMouseDown = function () { trace("button2.onMouseDown"); }
button2.onMouseMove = function () { trace("button2.onMouseMove"); }
button2.onMouseUp =   function () { trace("button2.onMouseUp"); }
button2.onKillFocus = function () { trace("button2.onKillFocus"); }
button2.onSetFocus =  function () { trace("button2.onSetFocus"); }
button2.onRollOut =   function () { trace("button2.onRollOut"); }
button2.onRollOver =  function () { trace("button2.onRollOver"); }

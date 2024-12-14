var testStage = 0;
// stage 0 — no additional logic
// stage 1 — Selection.onSetFocus clip->button->text->button2->clip2

var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed: " + oldFocus + " -> " + newFocus);
    }

    if (testStage == 1) {
        if (newFocus == clip) {
            Selection.setFocus(button);
        }
        if (newFocus == button) {
            Selection.setFocus(text);
        }
        if (newFocus == text) {
            Selection.setFocus(button2);
        }
        if (newFocus == button2) {
            Selection.setFocus(clip2);
        }
    }
};
listener.onKeyDown = function() {
    if (Key.getCode() == 9) {
        trace("Tab pressed");
    }
    if (Key.getCode() == 27) {
        trace("===== Next test stage: " + ++testStage);
    }
};
Selection.addListener(listener);
Key.addListener(listener);

function setHandlers(obj) {
    obj.onRelease =        function () { trace(obj + ".onRelease"); }
    obj.onData =           function () { trace(obj + ".onData"); }
    obj.onDragOut =        function () { trace(obj + ".onDragOut"); }
    obj.onDragOver =       function () { trace(obj + ".onDragOver"); }
    obj.onKeyDown =        function () { trace(obj + ".onKeyDown"); }
    obj.onKeyUp =          function () { trace(obj + ".onKeyUp"); }
    obj.onLoad =           function () { trace(obj + ".onLoad"); }
    obj.onMouseDown =      function () { trace(obj + ".onMouseDown"); }
    obj.onMouseMove =      function () { trace(obj + ".onMouseMove"); }
    obj.onMouseUp =        function () { trace(obj + ".onMouseUp"); }
    obj.onPress =          function () { trace(obj + ".onPress"); }
    obj.onRelease =        function () { trace(obj + ".onRelease"); }
    obj.onReleaseOutside = function () { trace(obj + ".onReleaseOutside"); }
    obj.onUnload =         function () { trace(obj + ".onUnload"); }
    obj.onKillFocus =      function () { trace(obj + ".onKillFocus"); }
    obj.onSetFocus =       function () { trace(obj + ".onSetFocus"); }
    obj.onRollOut =        function () { trace(obj + ".onRollOut"); }
    obj.onRollOver =       function () { trace(obj + ".onRollOver"); }
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

button2.tabEnabled = true;
button2.tabIndex = 3;
setHandlers(button2);

clip2.tabEnabled = true;
clip2.tabIndex = 4;
setHandlers(clip2);


trace("===== Setting focus manually");
trace("Setting the focus to " + clip);
Selection.setFocus(clip);
trace("Focus set to " + clip);
trace("Setting the focus to " + button);
Selection.setFocus(button);
trace("Focus set to " + button);
trace("Setting the focus to " + text);
Selection.setFocus(text);
trace("Focus set to " + text);
trace("Setting the focus to " + button2);
Selection.setFocus(button2);
trace("Focus set to " + button2);
trace("Setting the focus to " + clip2);
Selection.setFocus(clip2);
trace("Focus set to " + clip2);
trace("===== Starting with stage 0");

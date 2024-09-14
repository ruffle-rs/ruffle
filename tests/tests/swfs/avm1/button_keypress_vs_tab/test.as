var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 9) {
        trace("Tab pressed");
    }
    if (Key.getCode() == 27) {
        trace("===== Escape pressed");

        button2._visible = false;
        button3._visible = true;
        Selection.setFocus(button1);
    }
};
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed: " + oldFocus + " -> " + newFocus);
    }
};
Key.addListener(listener);
Selection.addListener(listener);

button1.tabIndex = 1;
button2.tabIndex = 2;
button3.tabIndex = 3;

var buttons = [button1, button2, button3];
for (var i in buttons) {
    buttons[i].onKeyDown = function () { trace("button.onKeyDown: " + Key.getCode()); }
    buttons[i].onKeyUp = function () { trace("button.onKeyUp: " + Key.getCode()); }
    buttons[i].onPress = function () { trace("button.onPress: " + Key.getCode()); }
    buttons[i].onRelease = function () { trace("button.onRelease: " + Key.getCode()); }
}

button3._visible = false;
Selection.setFocus(button1);

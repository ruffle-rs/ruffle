var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 32) {
        trace("Space pressed");
    }
    if (Key.getCode() == 13) {
        trace("Enter pressed");
    }
    if (Key.getCode() == 9) {
        trace("Tab pressed");
    }
};
Key.addListener(listener);

button.onKeyDown = function () { trace("button.onKeyDown: " + Key.getCode()); }
button.onKeyUp = function () { trace("button.onKeyUp: " + Key.getCode()); }
button.onPress = function () { trace("button.onPress: " + Key.getCode()); }
button.onRelease = function () { trace("button.onRelease: " + Key.getCode()); }

button2.onKeyDown = function () { trace("button2.onKeyDown: " + Key.getCode()); }
button2.onKeyUp = function () { trace("button2.onKeyUp: " + Key.getCode()); }
button2.onPress = function () { trace("button2.onPress: " + Key.getCode()); }
button2.onRelease = function () { trace("button2.onRelease: " + Key.getCode()); }

Selection.setFocus(button);

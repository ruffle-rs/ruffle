var listener = new Object();
listener.onKeyDown = function() {
    trace("pressed: " + Key.getCode());
};
Key.addListener(listener);

button.onKeyDown = function () { trace("button.onKeyDown"); }
button.onKeyUp = function () { trace("button.onKeyUp"); }
button.onPress = function () { trace("button.onPress"); }
button.onRelease = function () { trace("button.onRelease"); }

text.onChanged = function () { trace("text.onChanged"); }

Selection.setFocus(text);

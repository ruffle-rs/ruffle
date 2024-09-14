// left right home end insert delete backspace tab enter up down pgup pgdown escape space

button.onKeyDown = function () { trace("button.onKeyDown: " + Key.getCode()); }
button.onKeyUp = function () { trace("button.onKeyUp: " + Key.getCode()); }
button.onPress = function () { trace("button.onPress: " + Key.getCode()); }
button.onRelease = function () { trace("button.onRelease: " + Key.getCode()); }

Selection.setFocus(button);

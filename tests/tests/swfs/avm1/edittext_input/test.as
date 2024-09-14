var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        trace(text.text);
    }
};
Key.addListener(listener);

Selection.setFocus(text);

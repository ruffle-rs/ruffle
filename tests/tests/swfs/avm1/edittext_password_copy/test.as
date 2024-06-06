var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        trace("Text: '" + text.text + "'");
    }
};
Key.addListener(listener);

text.password = true;

Selection.setFocus(text);

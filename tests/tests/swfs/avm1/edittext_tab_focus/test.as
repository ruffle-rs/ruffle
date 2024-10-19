var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        trace("Focus: " + Selection.getFocus());
        var before = text.text;
        text.replaceSel("|");
        trace("Selection: " + before + " -> " + text.text);
        text.text = "text";
    }
    if (Key.getCode() == 9) {
        trace("Tab pressed");
    }
};
Key.addListener(listener);

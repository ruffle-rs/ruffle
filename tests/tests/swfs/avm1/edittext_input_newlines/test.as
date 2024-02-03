function escapeNewlines(text) {
    return text
        .split("\r").join("\\r")
        .split("\n").join("\\n");
}

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        trace("text: " + escapeNewlines(text.text));
        trace("text multiline: " + escapeNewlines(textMultiline.text));
        Selection.setFocus(textMultiline);
    }
};
Key.addListener(listener);

text.onChanged = function () {
    trace("text.onChanged: " + escapeNewlines(text.text));
}
textMultiline.onChanged = function () {
    trace("textMultiline.onChanged: " + escapeNewlines(textMultiline.text));
}

Selection.setFocus(text);

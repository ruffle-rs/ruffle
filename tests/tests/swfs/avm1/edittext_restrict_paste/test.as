var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        trace("Text: '" + text.text + "'");
        text.text = "";
    }
};
Key.addListener(listener);

text.restrict = "bc";
text.maxChars = 3;

Selection.setFocus(text);

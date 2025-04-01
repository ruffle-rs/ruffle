_root.createTextField("text", _root.getNextHighestDepth(), 0, 0, 100, 100);
text.type = "input";
Selection.setFocus(text);
text.onChanged = function() {
    trace("Changed: " + text.text);
};

setTimeout(function() {
    trace("Before: " + text.text);
    trace("  " + text.length);
    text.replaceSel("|");
    trace("  " + text.text);

    Selection.setFocus(null);

    setTimeout(function() {
        trace("After: " + text.text);
        trace("  " + text.length);
        text.replaceSel("|");
        trace("  " + text.text);
    }, 1000);
}, 5000);

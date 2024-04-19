var counter = 0;
var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    ++counter;
    if (counter > 4) {
        return;
    }
    if (newFocus) {
        trace("Focus changed");
        trace("  old: " + oldFocus);
        trace("  new: " + newFocus);
    }
};
Selection.addListener(listener);

text.tabIndex = 4294967293;
text2.tabIndex = -2;
button.tabIndex = 0;

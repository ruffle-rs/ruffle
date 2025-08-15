var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed");
        trace("  old: " + oldFocus);
        trace("  new: " + newFocus);
    }
};
Selection.addListener(listener);

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        trace("Escape pressed");
        Selection.setFocus(text5);
    } else if (Key.getCode() == 9) {
        trace("Tab pressed");
    }
};
Key.addListener(listener);

text2.tabIndex = 2;
text3.tabIndex = 1;
text4.tabIndex = 4;
text4.tabEnabled = false;
text5.tabIndex = 3;
text6.tabIndex = 5;

Selection.setFocus(text1);

var stopTrace = false;
var tracedCount = 0;

var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus && !stopTrace) {
        trace("Focus changed");
        trace("  old: " + oldFocus);
        trace("  new: " + newFocus);
    }
};
Selection.addListener(listener);

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 9 && !stopTrace) {
        ++tracedCount;
        if (tracedCount == 9) {
            stopTrace = true;
        } else {
            trace("Tab pressed");
        }
    }
};
Key.addListener(listener);

text1.tabIndex = 1;
text2.tabIndex = 0;
text3.tabIndex = -1;
text4.tabIndex = 2;
text5.tabIndex = -2;
text6.tabIndex = -3;

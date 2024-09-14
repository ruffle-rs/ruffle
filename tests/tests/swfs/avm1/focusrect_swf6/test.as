var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed");
        trace("  old: " + oldFocus);
        trace("  new: " + newFocus);
    }
};
Selection.addListener(listener);

clip1.tabEnabled = true;
clip1.tabIndex = 1;
clip2.tabEnabled = true;
clip2.tabIndex = 2;

var testStage = 0;
function nextTestStage() {
    testStage += 1;
    trace("Setting test stage to " + testStage);
    if (testStage == 1) {
        _focusrect = true;
    } else if (testStage == 2) {
        _focusrect = false;
    } else if (testStage == 3) {
        _focusrect = true;
        clip1._focusrect = false;
    } else if (testStage == 4) {
        _focusrect = false;
        clip1._focusrect = true;
    } else if (testStage == 5) {
        _focusrect = true;
        clip1._focusrect = null;
    } else if (testStage == 6) {
        _focusrect = false;
        clip1._focusrect = null;
    }
    Selection.setFocus(clip2);
}

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        nextTestStage();
    }
};
Key.addListener(listener);

var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed");
        trace("  old: " + oldFocus);
        trace("  new: " + newFocus);
    }
};
Selection.addListener(listener);

var testStage = 0;
function setUpStage(stage) {
    if (stage == 0) {
        // already set up
    }
    if (stage == 1) {
        Selection.setFocus(text4);
    }
    if (stage == 2) {
        Selection.setFocus(text1);
        text2.tabEnabled = false;
    }
    if (stage == 3) {
        Selection.setFocus(text1);
        text1.tabEnabled = false;
        text2.tabEnabled = true;
    }
}

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        testStage += 1;
        trace("Escape pressed, moving to stage " + testStage);
        setUpStage(testStage);
    } else if (Key.getCode() == 9) {
        trace("Tab pressed");
    }
};
Key.addListener(listener);

Selection.setFocus(text1);

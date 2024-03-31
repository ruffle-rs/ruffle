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
        _root.tabChildren = false;
        clipOuter.tabChildren = true;
        clipOuter.clipInner.tabChildren = true;
    }
    if (stage == 2) {
        _root.tabChildren = true;
        clipOuter.tabChildren = false;
        clipOuter.clipInner.tabChildren = true;
    }
    if (stage == 3) {
        _root.tabChildren = true;
        clipOuter.tabChildren = true;
        clipOuter.clipInner.tabChildren = false;
    }
    if (stage == 4) {
        _root.tabChildren = false;
        clipOuter.tabChildren = true;
        clipOuter.clipInner.tabChildren = false;
    }
    if (stage == 5) {
        _root.tabChildren = true;
        clipOuter.tabChildren = false;
        clipOuter.tabEnabled = false;
        clipOuter.clipInner.tabChildren = true;
    }
    if (stage == 6) {
        _root.tabChildren = true;
        clipOuter.tabChildren = false;
        clipOuter.tabEnabled = true;
        clipOuter.clipInner.tabChildren = true;
    }
    if (stage == 7) {
        _root.tabChildren = true;
        _root.tabEnabled = false;
        clipOuter.tabChildren = false;
        clipOuter.clipInner.tabChildren = true;
    }
    if (stage == 8) {
        _root.tabChildren = true;
        _root.tabEnabled = undefined;
        clipOuter.tabChildren = false;
        clipOuter.tabEnabled = undefined;
        clipOuter.clipInner.tabChildren = true;

        text1.tabIndex = 3;
        clipOuter.text3.tabIndex = 1;
        clipOuter.clipInner.text5.tabIndex = 2;
    }
    if (stage == 9) {
        _root.tabChildren = true;
        clipOuter.tabChildren = true;
        clipOuter.clipInner.tabChildren = false;

        text1.tabIndex = 3;
        clipOuter.text3.tabIndex = 1;
        clipOuter.clipInner.text5.tabIndex = 2;
    }
    if (stage == 10) {
        _root.tabChildren = false;
        clipOuter.tabChildren = false;
        clipOuter.clipInner.tabChildren = true;

        text1.tabIndex = 3;
        clipOuter.text3.tabIndex = 1;
        clipOuter.clipInner.text5.tabIndex = 2;
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

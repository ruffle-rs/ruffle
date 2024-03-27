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
    Selection.setFocus(text1);

    var currentStage = 0;
    if (stage == currentStage++) { // 0
        // already set up
    } else if (stage == currentStage++) {
        // custom tab ordering
        clip.tabIndex = 2;
        text1.tabIndex = 1;
        clip.text2.tabIndex = 3;
    } else if (stage == currentStage++) {
        // explicit enable
        clip.tabIndex = undefined;
        text1.tabIndex = undefined;
        clip.text2.tabIndex = undefined;

        clip.tabEnabled = true;
    } else if (stage == currentStage++) {
        // explicit disable
        clip.tabEnabled = false;
    } else if (stage == currentStage++) {
        // implicit enable with undefined handler
        clip.tabEnabled = undefined;
        clip.onRelease = undefined;
    } else if (stage == currentStage++) { // 5
        // implicit enable with handler
        clip.onRelease = function () {}
    } else if (stage == currentStage++) {
        // after delete
        delete clip.onRelease;
    } else if (stage == currentStage++) {
        // other handlers
        clip.onData = function () {}
    } else if (stage == currentStage++) {
        delete clip.onData;
        clip.onDragOut = function () {}
    } else if (stage == currentStage++) {
        delete clip.onDragOut;
        clip.onDragOver = function () {}
    } else if (stage == currentStage++) { // 10
        delete clip.onDragOver;
        clip.onEnterFrame = function () {}
    } else if (stage == currentStage++) {
        delete clip.onEnterFrame;
        clip.onKeyDown = function () {}
    } else if (stage == currentStage++) {
        delete clip.onKeyDown;
        clip.onKeyUp = function () {}
    } else if (stage == currentStage++) {
        delete clip.onKeyUp;
        clip.onKillFocus = function () {}
    } else if (stage == currentStage++) {
        delete clip.onKillFocus;
        clip.onLoad = function () {}
    } else if (stage == currentStage++) { // 15
        delete clip.onLoad;
        clip.onMouseDown = function () {}
    } else if (stage == currentStage++) {
        delete clip.onMouseDown;
        clip.onMouseMove = function () {}
    } else if (stage == currentStage++) {
        delete clip.onMouseMove;
        clip.onMouseUp = function () {}
    } else if (stage == currentStage++) {
        delete clip.onMouseUp;
        clip.onPress = function () {}
    } else if (stage == currentStage++) {
        delete clip.onPress;
        clip.onRelease = function () {}
    } else if (stage == currentStage++) { // 20
        delete clip.onRelease;
        clip.onReleaseOutside = function () {}
    } else if (stage == currentStage++) {
        delete clip.onReleaseOutside;
        clip.onRollOut = function () {}
    } else if (stage == currentStage++) {
        delete clip.onRollOut;
        clip.onRollOver = function () {}
    } else if (stage == currentStage++) {
        delete clip.onRollOver;
        clip.onSetFocus = function () {}
    } else if (stage == currentStage++) {
        delete clip.onSetFocus;
        clip.onUnload = function () {}
    } else { // 25
        trace("Finished!");
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

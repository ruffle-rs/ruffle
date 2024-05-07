clip.tabEnabled = true;
clip.tabIndex = 1;

var testStage = 0;
var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        testStage++;
        if (testStage == 1) {
            clip._x += 10;
            clip._y += 10;
        }
        if (testStage == 2) {
            clip._xscale = 50;
            clip._yscale = 50;
        }
        if (testStage == 3) {
            clip._xscale = 100;
            clip._yscale = 100;
            clip.inner._xscale = 200;
        }
        if (testStage == 4) {
            clip._focusrect = false;
        }
    }
};
Key.addListener(listener);

button1.tabIndex = 1;
button2.tabIndex = 2;
button3.tabIndex = 3;
button4.tabIndex = 4;
button5.tabIndex = 5;
clip6.tabIndex = 6;
clip6.tabEnabled = true;
clip6.focusEnabled = true;

var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed: " + oldFocus + " -> " + newFocus);
    }
};
Selection.addListener(listener);

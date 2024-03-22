clip.tabEnabled = true;
clip.focusEnabled = true;
clip.tabIndex = 1;
clip2.tabEnabled = true;
clip2.focusEnabled = true;
clip2.tabIndex = 2;

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        Selection.setFocus(clip2);
        Selection.setFocus(clip);
    }
};
Key.addListener(listener);

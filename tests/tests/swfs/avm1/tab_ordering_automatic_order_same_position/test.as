var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed: " + oldFocus + " -> " + newFocus);
    }
};
Selection.addListener(listener);

clip1a.tabEnabled = true;
clip1b.tabEnabled = true;
clip2.tabEnabled = true;
clip3.tabEnabled = true;
clip4.tabEnabled = true;

clip2.clipInner.tabEnabled = true;
clip3.clipInner1.tabEnabled = true;
clip3.clipInner2.tabEnabled = true;
clip4.clipInner1.tabEnabled = true;
clip4.clipInner2.tabEnabled = true;

clip5a.tabEnabled = true;
clip5b.tabEnabled = true;
clip6a.tabEnabled = true;
clip6b.tabEnabled = true;

var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        trace("Focus changed: " + oldFocus + " -> " + newFocus);
    }
};
Selection.addListener(listener);

clip1.tabEnabled = true;
clip2.tabEnabled = true;
clip3.tabEnabled = true;
clip4.tabEnabled = true;
clip5.tabEnabled = true;
clip6.tabEnabled = true;
clip7.tabEnabled = true;
clip8.tabEnabled = true;
clip9.tabEnabled = true;
clip10.tabEnabled = true;
clip11.tabEnabled = true;
clip12.tabEnabled = true;
clip13.tabEnabled = true;
clip14.tabEnabled = true;
clip15.tabEnabled = true;
clip16.tabEnabled = true;
clip17.tabEnabled = true;
clip18.tabEnabled = true;
clip19.tabEnabled = true;
clip20.tabEnabled = true;

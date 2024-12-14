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
outer.tabEnabled = true;
outer.clip11.tabEnabled = true;
outer.clip12.tabEnabled = true;
outer.clip13.tabEnabled = true;
outer.clip14.tabEnabled = true;
outer.clip15.tabEnabled = true;
outer.clip16.tabEnabled = true;
outer.clip17.tabEnabled = true;
outer.clip18.tabEnabled = true;
outer.clip19.tabEnabled = true;
outer.clip20.tabEnabled = true;

clip1.tabIndex = 2;
clip2.tabIndex = 2;
clip3.tabIndex = 2;
clip4.tabIndex = 2;
clip5.tabIndex = 2;
clip6.tabIndex = 2;
clip7.tabIndex = 2;
clip8.tabIndex = 2;
clip9.tabIndex = 2;
clip10.tabIndex = 2;
outer.tabIndex = 2;
outer.clip11.tabIndex = 2;
outer.clip12.tabIndex = 2;
outer.clip13.tabIndex = 2;
outer.clip14.tabIndex = 2;
outer.clip15.tabIndex = 2;
outer.clip16.tabIndex = 2;
outer.clip17.tabIndex = 2;
outer.clip18.tabIndex = 2;
outer.clip19.tabIndex = 2;
outer.clip20.tabIndex = 2;

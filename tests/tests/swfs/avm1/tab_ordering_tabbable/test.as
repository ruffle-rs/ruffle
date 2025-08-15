var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    if (newFocus) {
        tabbedObjects.push(newFocus);
    }
};
Selection.addListener(listener);

text1.type = "dynamic";
text2.maxChars = 0;
text3._visible = false;
text6.selectable = false;
clip9._visible = false;
button11._visible = false;
text12.type = "dynamic";
text12.tabEnabled = true;

var objects = [
    text1,
    text2,
    text3,
    text4,
    text5,
    text6,
    text7,
    clip8,
    clip8.text,
    clip9,
    clip9.text,
    button10,
    button11,
    text12
];

var tabbedObjects = [];

var testStage = 0;

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        trace("Tabbable elements:");
        for (var i in objects) {
            var exists = false;
            for (var j in tabbedObjects) {
                if (objects[i] == tabbedObjects[j]) {
                    exists = true;
                }
            }
            trace("  " + objects[i] + ": " + exists);
        }

        ++testStage;
        if (testStage == 1) {
            trace("Enabling tab");
            for (var i in objects) {
                objects[i].tabEnabled = true;
            }
        } else if (testStage == 2) {
            trace("Setting custom order");
            for (var i in objects) {
                objects[i].tabIndex = i;
            }
        }
    }
};
Key.addListener(listener);

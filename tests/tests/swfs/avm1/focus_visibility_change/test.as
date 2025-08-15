var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    trace("Focus changed");
    trace("  old: " + oldFocus);
    trace("  new: " + newFocus);
};
Selection.addListener(listener);

function testObject(obj) {
    trace("Object: " + obj);
    obj._visible = true;
    trace("Made visible");
    obj.focusEnabled = true;
    Selection.setFocus(obj);
    trace("Focus: " + Selection.getFocus());
    obj._visible = false;
    trace("Made invisible");
    trace("Focus: " + Selection.getFocus());

    obj._visible = true;
    trace("Made visible");
    trace("Focus: " + Selection.getFocus());
    obj._visible = false;
    trace("Made invisible");
}

var clip = _root.createEmptyMovieClip("clip", 10);
var text = _root.createTextField("text", 11, 0, 0, 150, 20);
text.type = "input";
var button = _root.attachMovie("CustomButton", "button", 12);

trace("===== clip");
testObject(clip);

trace("===== text");
testObject(text);

trace("===== button");
testObject(button);

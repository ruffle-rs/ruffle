var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    trace("Focus changed");
    trace("  old: " + oldFocus);
    trace("  new: " + newFocus);
};
Selection.addListener(listener);

function testObject(create, remove) {
    var obj = create();
    trace("Object: " + obj);
    obj.focusEnabled = true;
    Selection.setFocus(obj);
    trace("Focus: " + Selection.getFocus());
    remove();
    trace("Focus: " + Selection.getFocus());

    create();
    trace("Focus: " + Selection.getFocus());
    remove();
}

trace("===== clip");
testObject(function() {
    return _root.createEmptyMovieClip("clip", 2);
}, function() {
    _root.clip.removeMovieClip();
});

trace("===== text");
testObject(function() {
    var mc = _root.createEmptyMovieClip("clip", 3);
    var tf = mc.createTextField("text", 1, 0, 0, 150, 20);
    tf.type = "input";
    return tf;
}, function() {
    _root.clip.removeMovieClip();
});

trace("===== button");
testObject(function() {
    var mc = _root.attachMovie("CustomButton", "clip", 3);
    return mc.button;
}, function() {
    _root.clip.removeMovieClip();
});

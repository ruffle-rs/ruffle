var currentColor = 0;
var colors = [
    0xFF0000,
    0x00FF00,
    0x0000FF,
    0x00FFFF,
    0xFF00FF,
    0xFFFF00
];

var objectId = 0;
var objects = [];

function nextColor() {
    return colors[currentColor++ % 6];
}

function newMovieClip(tabEnabled, enabled, buttonMode, handCursor, focusEnabled) {
    var mc = _root.createEmptyMovieClip(
            "clip" + (objectId++),
            _root. getNextHighestDepth()
    );
    var color:int = nextColor();
    mc.enabled = enabled;
    if (buttonMode) {
        mc.onPress = function(){};
    }
    mc.useHandCursor = handCursor;
    mc.beginFill(color);
    mc.moveTo(0, 0);
    mc.lineTo(100, 0);
    mc.lineTo(100, 100);
    mc.lineTo(0, 100);
    mc.lineTo(0, 0);
    mc.endFill();
    mc.tabEnabled = tabEnabled;
    mc.focusEnabled = focusEnabled;
    return mc;
}

function newTextField(tabEnabled, input, selectable) {
    var tf = _root.createTextField(
            "text" + (objectId++),
            _root. getNextHighestDepth(),
            0, 0, 100, 100
    );
    tf.type = input ? "input" : "dynamic";
    tf.border = true;
    tf.selectable = selectable;
    var color:int = nextColor();
    tf.borderColor = color;
    tf.tabEnabled = tabEnabled;
    return tf;
}

objects.push(newTextField(false, false, false)); // 0
objects.push(newTextField(false, false, true));
objects.push(newTextField(false, true, false));
objects.push(newTextField(false, true, true));
objects.push(newTextField(true, false, false));
objects.push(newTextField(true, false, true));
objects.push(newTextField(true, true, false));
objects.push(newTextField(true, true, true));
objects.push(newMovieClip(false, false, false, false, false));
objects.push(newMovieClip(false, false, false, false, true));
objects.push(newMovieClip(false, false, false, true, false)); // 10
objects.push(newMovieClip(false, false, false, true, true));
objects.push(newMovieClip(false, false, true, false, false));
objects.push(newMovieClip(false, false, true, false, true));
objects.push(newMovieClip(false, false, true, true, false));
objects.push(newMovieClip(false, false, true, true, true));
objects.push(newMovieClip(false, true, false, false, false));
objects.push(newMovieClip(false, true, false, false, true));
objects.push(newMovieClip(false, true, false, true, false));
objects.push(newMovieClip(false, true, false, true, true));
objects.push(newMovieClip(false, true, true, false, false)); // 20
objects.push(newMovieClip(false, true, true, false, true));
objects.push(newMovieClip(false, true, true, true, false));
objects.push(newMovieClip(false, true, true, true, true));
objects.push(newMovieClip(true, false, false, false, false));
objects.push(newMovieClip(true, false, false, false, true));
objects.push(newMovieClip(true, false, false, true, false));
objects.push(newMovieClip(true, false, false, true, true));
objects.push(newMovieClip(true, false, true, false, false));
objects.push(newMovieClip(true, false, true, false, true));
objects.push(newMovieClip(true, false, true, true, false)); // 30
objects.push(newMovieClip(true, false, true, true, true));
objects.push(newMovieClip(true, true, false, false, false));
objects.push(newMovieClip(true, true, false, false, true));
objects.push(newMovieClip(true, true, false, true, false));
objects.push(newMovieClip(true, true, false, true, true));
objects.push(newMovieClip(true, true, true, false, false));
objects.push(newMovieClip(true, true, true, false, true));
objects.push(newMovieClip(true, true, true, true, false));
objects.push(newMovieClip(true, true, true, true, true));

var x = 0;
var y = 100;
for (var i in objects) {
    var object = objects[i];
    object._x = x;
    object._y = y;

    x += 100;
    if (x >= 1000) {
        x = 0;
        y += 100;
    }
}

var listener = new Object();
listener.onSetFocus = function(oldFocus, newFocus) {
    trace("Focus changed: " + oldFocus + " -> " + newFocus);
};
Selection.addListener(listener);

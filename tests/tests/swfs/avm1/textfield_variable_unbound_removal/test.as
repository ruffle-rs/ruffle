// Flash rebinds a text field whose bound clip was removed through that clip's
// path, without running ActionScript. Ruffle used to re-resolve the variable
// path instead, which runs getters, and removing a text field from a getter
// made it index past the end of its list of unbound text fields.

var armed = false;

_root.createEmptyMovieClip("clipA", 1);
_root.createEmptyMovieClip("clipB", 2);
_root.addProperty("holderA", function() {
    trace("get holderA");
    if (armed) {
        armed = false;
        trace("removing b");
        b.removeTextField();
    }
    return clipA;
}, null);

_root.createTextField("a", 3, 0, 0, 100, 20);
a.variable = "holderA.value";
_root.createTextField("b", 4, 0, 30, 100, 20);
b.variable = "clipB.value";

clipA.removeMovieClip();
clipB.removeMovieClip();

armed = true;
_root.createTextField("c", 5, 0, 60, 100, 20);
trace("b: " + b);

_root.createEmptyMovieClip("clipA", 6);
clipA.value = "new A";
_root.createEmptyMovieClip("clipB", 7);
clipB.value = "new B";

var frames = 0;
_root.onEnterFrame = function() {
    frames++;
    if (frames == 2) {
        trace("a: " + a.text);
        trace("b: " + b.text);
        delete _root.onEnterFrame;
    }
};

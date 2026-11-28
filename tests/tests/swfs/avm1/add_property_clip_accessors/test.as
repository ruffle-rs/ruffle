// Tests Object.addProperty with a movie clip as the getter and as the
// setter, and Object.registerClass with a movie clip as the constructor.
// It traces the return values and the property values afterwards.

createEmptyMovieClip("clip", getNextHighestDepth());

var object = {};
var stored = "initial";

function getter() {
    return stored;
}

function setter(value) {
    stored = value;
}

trace("addProperty(\"a\", clip, null): " + object.addProperty("a", clip, null));
trace("object.a: " + object.a);

trace("addProperty(\"b\", clip, setter): " + object.addProperty("b", clip, setter));
trace("object.b: " + object.b);

trace("addProperty(\"c\", getter, clip): " + object.addProperty("c", getter, clip));
object.c = "assigned";
trace("object.c: " + object.c + ", stored: " + stored);

trace("registerClass(\"symbol\", clip): " + Object.registerClass("symbol", clip));

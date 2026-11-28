// Tests Array.sortOn with a movie clip as the field names and as the
// options.
//
// The clip `fields` is at `_level0.fields`, has `length = 1` and `fields[0]
// = "x"`. Each element has a field `x`, a field named `_level0.fields`, and
// a `toString` method that returns its name; each of these orders the
// elements differently.

createEmptyMovieClip("fields", getNextHighestDepth());
fields.length = 1;
fields[0] = "x";

function element(name, x, pathField) {
    var object = {x: x};
    object["_level0.fields"] = pathField;
    object.toString = function() {
        return name;
    };
    return object;
}

function makeArray() {
    return [element("B", 3, 1), element("A", 2, 3), element("C", 1, 2)];
}

trace("x: C, A, B; _level0.fields: B, C, A; toString: A, B, C");

var array = makeArray();
var result = array.sortOn(fields);
trace("sortOn(clip): " + result + " / " + array);

var array = makeArray();
var result = array.sortOn(fields, Array.DESCENDING);
trace("sortOn(clip, DESCENDING): " + result + " / " + array);

var array = makeArray();
var result = array.sortOn(["x"], fields);
trace("sortOn([\"x\"], clip): " + result + " / " + array);

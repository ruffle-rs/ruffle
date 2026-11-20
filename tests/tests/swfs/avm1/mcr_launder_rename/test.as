// A reference to a clip stays tied to that clip while it exists. Once the
// clip is removed, the reference becomes path-based: it refers to whatever
// clip is currently at its path.
//
// This tests that passing a path-based reference to a function keeps it
// path-based, instead of tying it to the clip it currently resolves to.

function ident(x) {
    return x;
}

// Creates a clip, removes it, and creates a new one with the same name and
// depth. Returns a reference taken from the first clip, which is now
// path-based.
function pathBasedReference(name, depth) {
    createEmptyMovieClip(name, depth);
    var ref = _root[name];
    _root[name].removeMovieClip();
    createEmptyMovieClip(name, depth);
    return ref;
}

// A clip at a lower depth takes over the name.
trace("// Another clip takes the name");
var ref = ident(pathBasedReference("a", 10));
ref._x = 100;
trace("a._x: " + a._x);
createEmptyMovieClip("a", 9);
a._x = 50;
trace("ref._x: " + ref._x);
trace("");

trace("// The clip is renamed");
var ref = ident(pathBasedReference("b", 3));
ref._x = 5;
b._name = "b_renamed";
trace("ref: " + ref);
trace("ref._x: " + ref._x);
trace("b_renamed._x: " + b_renamed._x);
trace("");

trace("// Other references to the renamed clip still follow it");
var ref = ident(pathBasedReference("c", 4));
ref._x = 7;
var fresh = c;
c._name = "c_renamed";
c_renamed.getThis = function() {
    return this;
};
trace("fresh: " + fresh + ", _x: " + fresh._x);
trace("c_renamed: " + c_renamed + ", _x: " + c_renamed._x);
trace("getInstanceAtDepth(4): " + getInstanceAtDepth(4));
trace("this: " + c_renamed.getThis());
trace("");

trace("// A removed clip passed to a function");
var removed = createEmptyMovieClip("d", 5);
removeMovieClip(removed);
function describe(clip) {
    trace("clip == undefined: " + (clip == undefined) + ", typeof: " + typeof(clip));
}
describe(removed);
trace("");

// In each of these cases, a clip with the same name is created at a lower
// depth, so the name now refers to it. A path-based reference follows the
// name, so it reports the new depth.
trace("// Passed to a function");
function checkDepth(ref) {
    trace("before: " + ref.getDepth());
    createEmptyMovieClip("e", 106);
    trace("after: " + ref.getDepth());
}
checkDepth(pathBasedReference("e", 206));
trace("");

trace("// Returned from a function");
var ref = ident(pathBasedReference("f", 207));
trace("before: " + ref.getDepth());
createEmptyMovieClip("f", 107);
trace("after: " + ref.getDepth());
trace("");

trace("// Stored in an object property");
var holder = {};
holder.ref = pathBasedReference("g", 208);
holder.ref = ident(holder.ref);
trace("before: " + holder.ref.getDepth());
createEmptyMovieClip("g", 108);
trace("after: " + holder.ref.getDepth());
trace("");

trace("// Not passed anywhere");
var ref = pathBasedReference("h", 209);
trace("before: " + ref.getDepth());
createEmptyMovieClip("h", 109);
trace("after: " + ref.getDepth());

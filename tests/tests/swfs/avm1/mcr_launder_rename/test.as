// Tests which clip a reference refers to after it was passed to a function.
//
// Each case takes a reference to a clip, removes the clip and creates a new
// one with the same name and depth, and passes the old reference through a
// function. Then it changes which clip the name refers to (by renaming the
// clip or by creating another clip with that name at a lower depth) and
// traces properties of the reference to show which clip it refers to.

function ident(x) {
    return x;
}

// Creates a clip, removes it, and creates a new one with the same name and
// depth. Returns a reference taken from the first clip.
function staleReference(name, depth) {
    createEmptyMovieClip(name, depth);
    var ref = _root[name];
    _root[name].removeMovieClip();
    createEmptyMovieClip(name, depth);
    return ref;
}

// A clip at a lower depth takes over the name.
trace("// Another clip takes the name");
var ref = ident(staleReference("a", 10));
ref._x = 100;
trace("a._x: " + a._x);
createEmptyMovieClip("a", 9);
a._x = 50;
trace("ref._x: " + ref._x);
trace("");

trace("// The clip is renamed");
var ref = ident(staleReference("b", 3));
ref._x = 5;
b._name = "b_renamed";
trace("ref: " + ref);
trace("b_renamed._x: " + b_renamed._x);
trace("");

trace("// Other references to the renamed clip still follow it");
var ref = ident(staleReference("c", 4));
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
// depth, and the depth of the reference is traced before and after.
trace("// Passed to a function");
function checkDepth(ref) {
    trace("before: " + ref.getDepth());
    createEmptyMovieClip("e", 106);
    trace("after: " + ref.getDepth());
}
checkDepth(staleReference("e", 206));
trace("");

trace("// Returned from a function");
var ref = ident(staleReference("f", 207));
trace("before: " + ref.getDepth());
createEmptyMovieClip("f", 107);
trace("after: " + ref.getDepth());
trace("");

trace("// Stored in an object property");
var holder = {};
holder.ref = staleReference("g", 208);
holder.ref = ident(holder.ref);
trace("before: " + holder.ref.getDepth());
createEmptyMovieClip("g", 108);
trace("after: " + holder.ref.getDepth());
trace("");

trace("// Not passed anywhere");
var ref = staleReference("h", 209);
trace("before: " + ref.getDepth());
createEmptyMovieClip("h", 109);
trace("after: " + ref.getDepth());

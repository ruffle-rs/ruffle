// Tests which clip a reference refers to after it went through a function
// call, an array or another value.
//
// Each case creates a clip, takes `staleRef` from it, removes it and creates
// a new clip with the same name, and takes `freshRef` from the new clip. It
// passes the references through the tested construct, then renames the new
// clip to `<name>_moved` (with `_x = 3`) and creates another clip with the
// old name (with `_x = 9`). The traced name and `_x` show which of the two
// clips each result refers to.

var depth = 1;

function makeStaleRef(name) {
    createEmptyMovieClip(name, depth);
    var staleRef = _root[name];
    _root[name].removeMovieClip();
    createEmptyMovieClip(name, depth);
    depth++;
    return staleRef;
}

function moveAway(name) {
    var movedName = name + "_moved";
    _root[name]._name = movedName;
    _root[movedName]._x = 3;
    createEmptyMovieClip(name, depth);
    depth++;
    _root[name]._x = 9;
}

function show(label, ref) {
    trace(label + ": " + ref + ", _x: " + ref._x);
}

function pair(a, b) {
    return {a: a, b: b};
}

function firstArgument() {
    return arguments[0];
}

function ident(x) {
    return x;
}

// Both references in one call.
var staleRef = makeStaleRef("p1");
var freshRef = p1;
var result = pair(staleRef, freshRef);
moveAway("p1");
show("pair(stale, fresh).a", result.a);
show("pair(stale, fresh).b", result.b);
show("staleRef", staleRef);
show("freshRef", freshRef);

var staleRef = makeStaleRef("p2");
var freshRef = p2;
var result = pair(freshRef, staleRef);
moveAway("p2");
show("pair(fresh, stale).a", result.a);
show("pair(fresh, stale).b", result.b);

// `this` in a method called on the stale reference.
var staleRef = makeStaleRef("p3");
p3.getThis = function() {
    return this;
};
var result = staleRef.getThis();
moveAway("p3");
show("staleRef.getThis()", result);
show("staleRef", staleRef);

// valueOf
var staleRef = makeStaleRef("p4");
var result = staleRef.valueOf();
moveAway("p4");
show("staleRef.valueOf()", result);

// arguments
var staleRef = makeStaleRef("p5");
var result = firstArgument(staleRef);
moveAway("p5");
show("arguments[0]", result);

// Function.call and Function.apply
var staleRef = makeStaleRef("p6");
var result = ident.call(null, staleRef);
moveAway("p6");
show("ident.call(null, staleRef)", result);

var staleRef = makeStaleRef("p7");
var result = ident.apply(null, [staleRef]);
moveAway("p7");
show("ident.apply(null, [staleRef])", result);

// Array literals and Array methods.
var staleRef = makeStaleRef("p8");
var result = [staleRef].concat([])[0];
moveAway("p8");
show("[staleRef].concat([])[0]", result);

var staleRef = makeStaleRef("p9");
var array = [staleRef];
array.reverse();
var result = array[0];
moveAway("p9");
show("[staleRef].reverse()[0]", result);

var staleRef = makeStaleRef("p10");
var result = [].concat(staleRef)[0];
moveAway("p10");
show("[].concat(staleRef)[0]", result);

var staleRef = makeStaleRef("p11");
var array = [];
array.push(staleRef);
var result = array[0];
moveAway("p11");
show("push(staleRef)", result);

var staleRef = makeStaleRef("p12");
var array = [];
array.unshift(staleRef);
var result = array[0];
moveAway("p12");
show("unshift(staleRef)", result);

var staleRef = makeStaleRef("p13");
var array = [];
array.splice(0, 0, staleRef);
var result = array[0];
moveAway("p13");
show("splice(0, 0, staleRef)", result);

var staleRef = makeStaleRef("p14");
var array = new Array(staleRef);
var result = array[0];
moveAway("p14");
show("new Array(staleRef)", result);

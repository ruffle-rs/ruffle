// Each case: `stale` was taken before its clip was removed and recreated
// (path-based), `fresh` is taken from the recreated clip (cached).
// Afterwards the clip is renamed to N + "_moved" (_x = 3) and a new clip
// takes the name N (_x = 9). Path-based refs see 9, cached refs see 3.
var depth = 1;
function setup(n) {
    createEmptyMovieClip(n, depth);
    var stale = _root[n];
    _root[n].removeMovieClip();
    createEmptyMovieClip(n, depth);
    depth++;
    return stale;
}
function move(n) {
    _root[n]._name = n + "_moved";
    _root[n + "_moved"]._x = 3;
    createEmptyMovieClip(n, depth);
    depth++;
    _root[n]._x = 9;
}
function show(label, r) { trace(label + ": " + r + " _x=" + r._x); }
function pair(a, b) { return {a: a, b: b}; }
function first() { return arguments[0]; }
function ident(x) { return x; }

var s1 = setup("p1"); var f1 = p1;
var o1 = pair(s1, f1); move("p1");
show("pair stale", o1.a); show("pair fresh", o1.b); show("orig stale", s1); show("orig fresh", f1);

var s2 = setup("p2"); var f2 = p2;
var o2 = pair(f2, s2); move("p2");
show("pair2 fresh", o2.a); show("pair2 stale", o2.b);

var s3 = setup("p3");
p3.self = function() { return this; };
var t3 = s3.self(); move("p3");
show("this via stale", t3); show("stale", s3);

var s4 = setup("p4");
var v4 = s4.valueOf(); move("p4");
show("valueOf via stale", v4);

var s5 = setup("p5");
var c5 = [s5].concat([])[0]; move("p5");
show("concat via stale", c5);

var s6 = setup("p6");
var a6 = first(s6); move("p6");
show("arguments via stale", a6);

var s7 = setup("p7");
var k7 = ident.call(null, s7); move("p7");
show("call via stale", k7);

var s8 = setup("p8");
var k8 = ident.apply(null, [s8]); move("p8");
show("apply via stale", k8);

var s9 = setup("p9");
var arr9 = [s9]; arr9.reverse(); var k9 = arr9[0]; move("p9");
show("array native via stale", k9);

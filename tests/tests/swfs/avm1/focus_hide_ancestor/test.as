// Hiding the focused object or one of its ancestors drops the focus.

function describe(o) {
    return o == null ? "null" : String(o);
}

function makeClip(parent, name, depth) {
    var mc = parent.createEmptyMovieClip(name, depth);
    mc.onRelease = function() {};
    mc.onSetFocus = function(old) { trace("  onSetFocus " + this + ", old: " + describe(old)); };
    mc.onKillFocus = function(newFocus) { trace("  onKillFocus " + this + ", new: " + describe(newFocus)); };
    return mc;
}

function makeText(parent, name, depth) {
    parent.createTextField(name, depth, 0, 0, 50, 20);
    var t = parent[name];
    t.type = "input";
    t.onSetFocus = function(old) { trace("  onSetFocus " + this + ", old: " + describe(old)); };
    t.onKillFocus = function(newFocus) { trace("  onKillFocus " + this + ", new: " + describe(newFocus)); };
    return t;
}

function focus(o) {
    Selection.setFocus(o);
    trace("Focus set: " + describe(Selection.getFocus()));
}

function after() {
    trace("After: " + describe(Selection.getFocus()));
}

trace("===== text field hidden");
var p = this.createEmptyMovieClip("p1", 1);
var t = makeText(p, "t", 1);
focus(t);
t._visible = false;
after();
Selection.setFocus(null);

trace("===== parent of a text field hidden");
p = this.createEmptyMovieClip("p2", 2);
t = makeText(p, "t", 1);
focus(t);
p._visible = false;
after();
Selection.setFocus(null);

trace("===== grandparent of a text field hidden");
var g = this.createEmptyMovieClip("g3", 3);
p = g.createEmptyMovieClip("p", 1);
t = makeText(p, "t", 1);
focus(t);
g._visible = false;
after();
Selection.setFocus(null);

trace("===== clip hidden");
p = this.createEmptyMovieClip("p4", 4);
var c = makeClip(p, "c", 1);
focus(c);
c._visible = false;
after();
Selection.setFocus(null);

trace("===== parent of a clip hidden");
p = this.createEmptyMovieClip("p5", 5);
c = makeClip(p, "c", 1);
focus(c);
p._visible = false;
after();
Selection.setFocus(null);

trace("===== text field hidden, focused, hidden again");
p = this.createEmptyMovieClip("p6", 6);
t = makeText(p, "t", 1);
t._visible = false;
focus(t);
t._visible = false;
after();
Selection.setFocus(null);

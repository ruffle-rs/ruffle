// Selection.getFocus() inside the focus handlers:
// onKillFocus still sees the old focus, onSetFocus and
// Selection.onSetFocus already see the new one.

function describe(o) {
    return o == null ? "null" : String(o);
}

function focus() {
    return describe(Selection.getFocus());
}

function addHandlers(o) {
    o.onSetFocus = function(oldFocus) {
        trace("  " + this._name + ".onSetFocus, old: " + describe(oldFocus) + ", focus: " + focus());
    };
    o.onKillFocus = function(newFocus) {
        trace("  " + this._name + ".onKillFocus, new: " + describe(newFocus) + ", focus: " + focus());
    };
}

function makeText(name, x, y, tabIndex) {
    this.createTextField(name, this.getNextHighestDepth(), x, y, 100, 30);
    var t = this[name];
    t.type = "input";
    t.border = true;
    t.tabIndex = tabIndex;
    addHandlers(t);
    return t;
}

function makeClip(name, x, y, tabIndex) {
    var c = this.createEmptyMovieClip(name, this.getNextHighestDepth());
    c.beginFill(0x00ff00);
    c.moveTo(0, 0);
    c.lineTo(100, 0);
    c.lineTo(100, 30);
    c.lineTo(0, 30);
    c.lineTo(0, 0);
    c.endFill();
    c._x = x;
    c._y = y;
    c.focusEnabled = true;
    c.tabIndex = tabIndex;
    addHandlers(c);
    return c;
}

var t1 = makeText("t1", 10, 10, 1);
var t2 = makeText("t2", 10, 60, 2);
var c1 = makeClip("c1", 10, 110, 3);
var c2 = makeClip("c2", 10, 160, 4);

var listener = {};
listener.onSetFocus = function(oldFocus, newFocus) {
    trace("  Selection.onSetFocus, " + describe(oldFocus) + " -> " + describe(newFocus) + ", focus: " + focus());
};
Selection.addListener(listener);

function setFocus(label, o) {
    trace("== " + label);
    Selection.setFocus(o);
    trace("After: " + focus());
}

setFocus("null -> t1", t1);
setFocus("t1 -> t2", t2);
setFocus("t2 -> c1", c1);
setFocus("c1 -> c2", c2);
setFocus("c2 -> t1", t1);
setFocus("t1 -> null", null);
setFocus("null -> c1", c1);
setFocus("c1 -> null", null);

trace("== focused text field hidden");
Selection.setFocus(t1);
t1._visible = false;
trace("After: " + focus());
t1._visible = true;

trace("== focused clip hidden");
Selection.setFocus(c1);
c1._visible = false;
trace("After: " + focus());
c1._visible = true;

trace("== focused text field removed");
var t3 = makeText("t3", 200, 10, 5);
Selection.setFocus(t3);
t3.removeTextField();
trace("After: " + focus());

trace("== focused clip removed");
var c3 = makeClip("c3", 200, 60, 6);
Selection.setFocus(c3);
c3.removeMovieClip();
trace("After: " + focus());

Selection.setFocus(null);

var mouseListener = {};
mouseListener.onMouseDown = function() {
    trace("== Mouse down at " + _root._xmouse + "," + _root._ymouse + ", focus: " + focus());
};
mouseListener.onMouseUp = function() {
    trace("Mouse up, focus: " + focus());
};
Mouse.addListener(mouseListener);

var keyListener = {};
keyListener.onKeyDown = function() {
    trace("== Key down " + Key.getCode() + ", focus: " + focus());
};
keyListener.onKeyUp = function() {
    trace("Key up " + Key.getCode() + ", focus: " + focus());
};
Key.addListener(keyListener);

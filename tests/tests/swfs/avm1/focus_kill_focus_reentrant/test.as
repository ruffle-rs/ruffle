// Focus handlers that change the focus while it is changing.
// When the new focus is a text field, Flash Player applies the last focus
// requested by a handler once the change is done (only for Selection.setFocus
// and tabbing, not for mouse clicks).

function describe(o) {
    return o == null ? "null" : String(o);
}

function focus() {
    return describe(Selection.getFocus());
}

var quiet = false;
var hook = {};

function log(s) {
    if (!quiet) {
        trace(s);
    }
}

function runHook(where, name) {
    if (hook.where == where && hook.name == name) {
        if (!hook.always) {
            hook.where = null;
        }
        log("  -> " + hook.description + " in " + name + "." + where);
        hook.action();
        log("  <- back in " + name + "." + where + ", focus: " + focus());
    }
}

function addHandlers(o, name) {
    o.onSetFocus = function(oldFocus) {
        log("  " + name + ".onSetFocus, old: " + describe(oldFocus) + ", focus: " + focus());
        runHook("onSetFocus", name);
    };
    o.onKillFocus = function(newFocus) {
        log("  " + name + ".onKillFocus, new: " + describe(newFocus) + ", focus: " + focus());
        runHook("onKillFocus", name);
    };
}

function makeText(name, x, y, tabIndex) {
    this.createTextField(name, this.getNextHighestDepth(), x, y, 100, 30);
    var t = this[name];
    t.type = "input";
    t.border = true;
    t.tabIndex = tabIndex;
    addHandlers(t, name);
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
    addHandlers(c, name);
    return c;
}

var t1 = makeText("t1", 10, 10, 1);
var t2 = makeText("t2", 10, 60, 2);
var t3 = makeText("t3", 200, 10, 7);
var c1 = makeClip("c1", 10, 110, 3);
var c2 = makeClip("c2", 10, 160, 4);
var c3 = makeClip("c3", 200, 60, 8);

var listener = {};
listener.onSetFocus = function(oldFocus, newFocus) {
    log("  Selection.onSetFocus, " + describe(oldFocus) + " -> " + describe(newFocus) + ", focus: " + focus());
    runHook("onSetFocus", "Selection");
};
Selection.addListener(listener);

function setHook(where, name, description, action, always) {
    hook.where = where;
    hook.name = name;
    hook.description = description;
    hook.action = action;
    hook.always = always;
}

function setFocusQuietly(o) {
    quiet = true;
    var where = hook.where;
    hook.where = null;
    Selection.setFocus(o);
    hook.where = where;
    quiet = false;
}

function run(label, from, to) {
    setFocusQuietly(from);
    trace("== " + label);
    var result = Selection.setFocus(to);
    hook.where = null;
    trace("Returned " + result + ", focus: " + focus());
}

function focusOn(target) {
    return function() {
        Selection.setFocus(target);
    };
}

setHook("onKillFocus", "t1", "setFocus(c1)", focusOn(c1));
run("t1 -> t2, t1.onKillFocus focuses c1", t1, t2);
setHook("onKillFocus", "t1", "setFocus(null)", focusOn(null));
run("t1 -> t2, t1.onKillFocus clears the focus", t1, t2);
setHook("onKillFocus", "t1", "setFocus(t2)", focusOn(t2));
run("t1 -> t2, t1.onKillFocus focuses t2", t1, t2);
setHook("onKillFocus", "t1", "setFocus(t3)", focusOn(t3));
run("t1 -> t2, t1.onKillFocus focuses t3", t1, t2);
setHook("onKillFocus", "t1", "setFocus(t1)", focusOn(t1));
run("t1 -> t2, t1.onKillFocus focuses t1", t1, t2);
setHook("onKillFocus", "c1", "setFocus(t1)", focusOn(t1));
run("c1 -> c2, c1.onKillFocus focuses t1", c1, c2);
setHook("onKillFocus", "c1", "setFocus(c1)", focusOn(c1));
run("c1 -> c2, c1.onKillFocus focuses c1", c1, c2);
setHook("onKillFocus", "c1", "setFocus(c3)", focusOn(c3));
run("c1 -> c2, c1.onKillFocus focuses c3", c1, c2);
setHook("onKillFocus", "c1", "setFocus(null)", focusOn(null));
run("c1 -> c2, c1.onKillFocus clears the focus", c1, c2);
setHook("onKillFocus", "t1", "setFocus(t2)", focusOn(t2));
run("t1 -> c1, t1.onKillFocus focuses t2", t1, c1);
setHook("onKillFocus", "t1", "setFocus(t1)", focusOn(t1));
run("t1 -> c1, t1.onKillFocus focuses t1", t1, c1);
setHook("onKillFocus", "c1", "setFocus(c2)", focusOn(c2));
run("c1 -> t1, c1.onKillFocus focuses c2", c1, t1);
setHook("onKillFocus", "c1", "setFocus(c1)", focusOn(c1));
run("c1 -> t1, c1.onKillFocus focuses c1", c1, t1);
setHook("onKillFocus", "t1", "setFocus(t2)", focusOn(t2));
run("t1 -> null, t1.onKillFocus focuses t2", t1, null);
setHook("onKillFocus", "t1", "setFocus(t1)", focusOn(t1));
run("t1 -> null, t1.onKillFocus focuses t1", t1, null);
setHook("onKillFocus", "c1", "setFocus(c2)", focusOn(c2));
run("c1 -> null, c1.onKillFocus focuses c2", c1, null);

setHook("onKillFocus", "t1", "setFocus(c1) and c1.onSetFocus clears the focus", function() {
    var onSetFocus = c1.onSetFocus;
    c1.onSetFocus = function(oldFocus) {
        log("  c1.onSetFocus (replaced), old: " + describe(oldFocus) + ", focus: " + focus());
        c1.onSetFocus = onSetFocus;
        Selection.setFocus(null);
    };
    Selection.setFocus(c1);
});
run("t1 -> t2, t1.onKillFocus focuses c1, which clears the focus", t1, t2);
setHook("onKillFocus", "t1", "setFocus(null), then setFocus(c1)", function() {
    Selection.setFocus(null);
    Selection.setFocus(c1);
});
run("t1 -> t2, t1.onKillFocus clears the focus, then focuses c1", t1, t2);
setHook("onKillFocus", "t1", "setFocus(c1), then setFocus(t1)", function() {
    Selection.setFocus(c1);
    Selection.setFocus(t1);
});
run("t1 -> t2, t1.onKillFocus focuses c1, then t1", t1, t2);

setHook("onSetFocus", "t2", "setFocus(t3)", focusOn(t3));
run("t1 -> t2, t2.onSetFocus focuses t3", t1, t2);
setHook("onSetFocus", "t2", "setFocus(null)", focusOn(null));
run("t1 -> t2, t2.onSetFocus clears the focus", t1, t2);
setHook("onSetFocus", "t2", "setFocus(t1)", focusOn(t1));
run("t1 -> t2, t2.onSetFocus focuses t1", t1, t2);
setHook("onSetFocus", "c2", "setFocus(null)", focusOn(null));
run("c1 -> c2, c2.onSetFocus clears the focus", c1, c2);
setHook("onSetFocus", "Selection", "setFocus(t3)", focusOn(t3));
run("t1 -> t2, Selection.onSetFocus focuses t3", t1, t2);
setHook("onSetFocus", "Selection", "setFocus(null)", focusOn(null));
run("c1 -> c2, Selection.onSetFocus clears the focus", c1, c2);
setHook("onSetFocus", "Selection", "setFocus(t1)", focusOn(t1));
run("t1 -> null, Selection.onSetFocus focuses t1", t1, null);

var r1 = makeText("r1", 200, 110, 9);
var r2 = makeText("r2", 200, 160, 10);
setHook("onKillFocus", "r1", "remove r2", function() { r2.removeTextField(); });
run("r1 -> r2, r1.onKillFocus removes r2", r1, r2);

var r3 = makeClip("r3", 200, 210, 11);
var r4 = makeClip("r4", 200, 260, 12);
setHook("onKillFocus", "r3", "remove r4", function() { r4.removeMovieClip(); });
run("r3 -> r4, r3.onKillFocus removes r4", r3, r4);

var r5 = makeText("r5", 200, 110, 13);
setHook("onKillFocus", "r5", "remove r5", function() { r5.removeTextField(); });
run("r5 -> t2, r5.onKillFocus removes r5", r5, t2);

var r6 = makeText("r6", 200, 160, 14);
setHook("onSetFocus", "r6", "remove r6", function() { r6.removeTextField(); });
run("t1 -> r6, r6.onSetFocus removes r6", t1, r6);

setHook("onKillFocus", "t1", "hide t2", function() { t2._visible = false; });
run("t1 -> t2, t1.onKillFocus hides t2", t1, t2);
t2._visible = true;

trace("== t1 and t2 both take the focus back in onKillFocus");
setFocusQuietly(t1);
var calls = 0;
var t1Kill = t1.onKillFocus;
var t2Kill = t2.onKillFocus;
t1.onKillFocus = function(newFocus) {
    trace("  t1.onKillFocus, new: " + describe(newFocus) + ", focus: " + focus());
    calls++;
    if (calls < 10) {
        Selection.setFocus(t1);
    }
};
t2.onKillFocus = function(newFocus) {
    trace("  t2.onKillFocus, new: " + describe(newFocus) + ", focus: " + focus());
    calls++;
    if (calls < 10) {
        Selection.setFocus(t2);
    }
};
var result = Selection.setFocus(t2);
trace("Returned " + result + ", focus: " + focus() + ", onKillFocus calls: " + calls);
t1.onKillFocus = t1Kill;
t2.onKillFocus = t2Kill;

setFocusQuietly(null);

var step = 0;
function nextStep() {
    step++;
    if (step == 1) {
        setFocusQuietly(t1);
        setHook("onKillFocus", "t1", "setFocus(null)", focusOn(null));
        trace("-- Next: Tab, t1.onKillFocus clears the focus");
    } else if (step == 2) {
        setFocusQuietly(t1);
        setHook("onKillFocus", "t1", "setFocus(c1)", focusOn(c1));
        trace("-- Next: Tab, t1.onKillFocus focuses c1");
    } else if (step == 3) {
        setFocusQuietly(t1);
        setHook("onKillFocus", "t1", "setFocus(t1)", focusOn(t1));
        trace("-- Next: Tab, t1.onKillFocus focuses t1");
    } else if (step == 4) {
        setFocusQuietly(t1);
        setHook("onKillFocus", "t1", "setFocus(t1)", focusOn(t1));
        trace("-- Next: click t2, t1.onKillFocus focuses t1");
    } else if (step == 5) {
        setFocusQuietly(t1);
        setHook("onKillFocus", "t1", "setFocus(null)", focusOn(null));
        trace("-- Next: click t2, t1.onKillFocus clears the focus");
    } else if (step == 6) {
        setFocusQuietly(t1);
        setHook("onKillFocus", "t1", "setFocus(t1)", focusOn(t1));
        trace("-- Next: click the stage, t1.onKillFocus focuses t1");
    } else {
        hook.where = null;
        setFocusQuietly(null);
    }
}

var mouseListener = {};
mouseListener.onMouseDown = function() {
    trace("== Mouse down at " + _root._xmouse + "," + _root._ymouse + ", focus: " + focus());
};
mouseListener.onMouseUp = function() {
    hook.where = null;
    trace("Mouse up, focus: " + focus());
    nextStep();
};
Mouse.addListener(mouseListener);

var keyListener = {};
keyListener.onKeyDown = function() {
    trace("== Key down " + Key.getCode() + ", focus: " + focus());
};
keyListener.onKeyUp = function() {
    hook.where = null;
    trace("Key up " + Key.getCode() + ", focus: " + focus());
    nextStep();
};
Key.addListener(keyListener);

setFocusQuietly(t1);
setHook("onKillFocus", "t1", "setFocus(c1)", focusOn(c1));
trace("-- Next: click t2, t1.onKillFocus focuses c1");

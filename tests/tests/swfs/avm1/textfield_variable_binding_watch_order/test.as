function makeChild() {
    var child = _root.createEmptyMovieClip("child", 10);
    child.watch("v1", function(p, o, n) { trace("watch v1: " + o + " -> " + n); return n; });
    child.watch("v2", function(p, o, n) { trace("watch v2: " + o + " -> " + n); return n; });
    child.watch("v3", function(p, o, n) { trace("watch v3: " + o + " -> " + n); return n; });
    return child;
}

var child = makeChild();

var tf1 = _root.createTextField("tf1", 1, 0, 0, 100, 20);
tf1.variable = "_root.child.v1";
var tf2 = _root.createTextField("tf2", 2, 0, 30, 100, 20);
tf2.variable = "_root.child.v2";
var tf3 = _root.createTextField("tf3", 3, 0, 60, 100, 20);
tf3.variable = "_root.child.v3";

trace("--- set child.vN (initial bind) ---");
child.v1 = "X";
child.v2 = "Y";
child.v3 = "Z";
trace("tf1.text=" + tf1.text + " tf2.text=" + tf2.text + " tf3.text=" + tf3.text);

trace("--- set tfN.text (propagate to vars) ---");
tf1.text = "P";
tf2.text = "Q";
tf3.text = "R";
trace("child.v1=" + child.v1 + " v2=" + child.v2 + " v3=" + child.v3);

trace("--- remove child (push tfs back to unbound list) ---");
child.removeMovieClip();
trace("child=" + child);

trace("--- recreate child (bind_variables reattaches tfs) ---");
child = makeChild();
trace("child.v1=" + child.v1 + " v2=" + child.v2 + " v3=" + child.v3);

trace("--- set child.vN again ---");
child.v1 = "A";
child.v2 = "B";
child.v3 = "C";
trace("tf1.text=" + tf1.text + " tf2.text=" + tf2.text + " tf3.text=" + tf3.text);

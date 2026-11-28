var BASE = -16000;
var D = [-15990, -15980, -15970, -15960, -15950, -15940, -15935, -15930, -15925, -15920, -15910, -15900, -15890, -15880, -15870, -15860, -15850, -15840, -15830, -15820, -15810, -15800, -15790, -15780, -15770, -15760, -15755, -15750, -15740, -15735, -15730, -15720, -15715, -15710, -15705, -15700, -15695, -15690, -15680, -15670, -15660, -15650, -15640, -15630, -15620, -15610, -15600, -15590, -15580, -15570, -15560, -15550];
var M = [-15980, -15930, -15910, -15900, -15890, -15880, -15840, -15820, -15800, -15750, -15720, -15710, -15700, -15590, -15580, -15570, -15560];
var DR = [-15460, -15450, -15440];
var K = [46, 47, 48, 49, 50, 51, 52, 53];
function desc(tl, o) {
    if (o == undefined) { return "-"; }
    if (o == tl) { return "(no object)"; }
    return o._name + " m=" + o.m + " x=" + o._x + " y=" + o._y + " a=" + o._alpha + " v=" + o.v;
}
function rep(tl, label) {
    for (var i = 0; i < D.length; i++) {
        trace(label + " d" + D[i] + ": " + desc(tl, tl.getInstanceAtDepth(D[i])));
    }
    for (var i = 0; i < K.length; i++) {
        var k = tl["k" + K[i]];
        var b = k.getBounds(k);
        var bs = b.xMin > 1000000 ? "empty" : (b.xMin + "," + b.yMin + " " + b.xMax + "," + b.yMax);
        trace(label + " k" + K[i] + " d" + (K[i] * 10 + BASE) + ": " + desc(k, k.getInstanceAtDepth(K[i] * 10 + BASE)) + " bounds=" + bs + " chars=" + k.getTextSnapshot().getCount());
    }
    repr(tl.r3, label + " r3");
    repr(tl.r4, label + " r4");
}
function repr(tl, label) {
    for (var i = 0; i < DR.length; i++) {
        trace(label + " d" + DR[i] + ": " + desc(tl, tl.getInstanceAtDepth(DR[i])));
    }
}
function mark(tl, f) {
    for (var i = 0; i < M.length; i++) {
        var o = tl.getInstanceAtDepth(M[i]);
        if (o != undefined && o.m == undefined) { o.m = "f" + f; }
    }
}

if (!_global.done) { trace("== frame 1"); rep(this, "f1"); mark(this, 1); }
else { trace("== frame 1 (after rewind)"); rep(this, "f1r"); }

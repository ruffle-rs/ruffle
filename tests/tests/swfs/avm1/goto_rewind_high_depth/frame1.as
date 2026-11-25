var BASE = 100;
var D = [110, 120, 130, 140, 150, 160, 165, 170, 175, 180, 190, 200, 210, 220, 230, 240, 250, 260, 270, 280, 290, 300, 310, 320, 330, 340, 345, 350, 360, 365, 370, 380, 385, 390, 395, 400, 405, 410, 420, 430, 440, 450, 460, 470, 480, 490, 500, 510, 520, 530, 540, 550];
var M = [120, 170, 190, 200, 210, 220, 260, 280, 300, 350, 380, 390, 400, 510, 520, 530, 540];
var DR = [640, 650, 660];
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

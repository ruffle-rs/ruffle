// A fully transparent shape (alpha 0, also through a parent) does not count as
// drawn, so hitTest(x, y, true) tests it where it is. Any other alpha counts,
// even a negative one.
function makeSquare(name, depth, parent) {
    if (parent == undefined) parent = _root;
    var mc = parent.createEmptyMovieClip(name, depth);
    mc.beginFill(0x808080);
    mc.moveTo(0, 0); mc.lineTo(100, 0); mc.lineTo(100, 100); mc.lineTo(0, 100); mc.lineTo(0, 0);
    mc.endFill();
    return mc;
}
function show(label, mc) {
    trace(label + ": shape(50,50)=" + mc.hitTest(50, 50, true) + " shape(250,50)=" + mc.hitTest(250, 50, true));
}
var a0 = makeSquare("a0", 1); a0._alpha = 0;
var a1 = makeSquare("a1", 2); a1._alpha = 1;
var pa = _root.createEmptyMovieClip("pa", 3); pa._alpha = 0;
var pc = makeSquare("pc", 1, pa);
var a0b = makeSquare("a0b", 4); a0b._alpha = 0;
var neg = makeSquare("neg", 5); neg._alpha = -50;
var ref = makeSquare("ref", 6);
var frame = 0;
onEnterFrame = function () {
    frame++;
    if (frame == 2) {
        ref._xscale = 300; show("reference (opaque), then _xscale=300", ref);
        a0._xscale = 300; show("_alpha=0, then _xscale=300", a0);
        a1._xscale = 300; show("_alpha=1, then _xscale=300", a1);
        pc._xscale = 300; show("parent _alpha=0, then _xscale=300", pc);
        a0b._alpha = 100; a0b._xscale = 300; show("_alpha=0, then _alpha=100 and _xscale=300", a0b);
        neg._xscale = 300; show("_alpha=-50, then _xscale=300", neg);
        delete onEnterFrame;
    }
};

// hitTest(x, y, true) on a clip whose drawing is changed from ActionScript
// after it was rendered. Each clip starts as a square from (0,0) to (100,100).
function square(mc, x) {
    mc.beginFill(0x808080);
    mc.moveTo(x, 0); mc.lineTo(x + 100, 0); mc.lineTo(x + 100, 100); mc.lineTo(x, 100); mc.lineTo(x, 0);
    mc.endFill();
}
function make(name, depth) {
    var mc = _root.createEmptyMovieClip(name, depth);
    square(mc, 0);
    return mc;
}
function show(label, mc) {
    trace(label + ": shape(50,50)=" + mc.hitTest(50, 50, true) + " shape(250,50)=" + mc.hitTest(250, 50, true) +
        " shape(350,50)=" + mc.hitTest(350, 50, true) + " bounds(50,50)=" + mc.hitTest(50, 50, false) +
        " bounds(250,50)=" + mc.hitTest(250, 50, false) + " bounds(350,50)=" + mc.hitTest(350, 50, false));
}

// Redrawn before it is ever rendered.
var a = make("a", 1);
a.clear();
square(a, 200);
show("never rendered, redrawn at x=200", a);

var b = make("b", 2);
var c = make("c", 3);
var d = make("d", 4);
var e = make("e", 5);
var frame = 0;
onEnterFrame = function () {
    frame++;
    if (frame == 2) {
        // All of them have been rendered as squares at x=0 now.
        b.clear();
        square(b, 200);
        show("rendered, then redrawn at x=200", b);
        c.clear();
        square(c, 200);
        c._x = 100;
        show("rendered, then redrawn at x=200 and _x=100", c);
        d.clear();
        show("rendered, then cleared", d);
        square(e, 200);
        show("rendered, then a second square added at x=200", e);
    }
    if (frame == 3) {
        show("next frame: b", b);
        show("next frame: d", d);
        delete onEnterFrame;
    }
};

// hitTest(x, y, true) tests a shape as it was last drawn, even if it was moved,
// scaled or rotated since. Shapes not drawn yet, hidden or off stage are tested
// where they are. Drawing into a bitmap counts as drawing.
// hitTest(x, y, false) always uses the current bounds.
// Each square goes from (0,0) to (100,100) in its own coordinates.
function makeSquare(name, depth, parent) {
    if (parent == undefined) parent = _root;
    var mc = parent.createEmptyMovieClip(name, depth);
    mc.beginFill(0x808080);
    mc.moveTo(0, 0); mc.lineTo(100, 0); mc.lineTo(100, 100); mc.lineTo(0, 100); mc.lineTo(0, 0);
    mc.endFill();
    return mc;
}
function show(label, mc) {
    trace(label + ": shape(50,50)=" + mc.hitTest(50, 50, true) + " shape(250,50)=" + mc.hitTest(250, 50, true) +
        " bounds(50,50)=" + mc.hitTest(50, 50, false) + " bounds(250,50)=" + mc.hitTest(250, 50, false));
}

// Changed before it is ever rendered.
var a = makeSquare("a", 1);
a._x = 200;
show("never rendered, moved to x=200", a);

var b = makeSquare("b", 2);
var c = makeSquare("c", 3);
var d = makeSquare("d", 4);
var e = makeSquare("e", 5);
var p = _root.createEmptyMovieClip("p", 6);
var child = makeSquare("child", 1, p);
var off = makeSquare("off", 7);
off._x = -1000;
var inv = makeSquare("inv", 8);
inv._visible = false;
var inv2 = makeSquare("inv2", 9);
var big = makeSquare("big", 10);
var bd = makeSquare("bd", 11);
var bmp = new flash.display.BitmapData(400, 400, true, 0);
var frame = 0;
onEnterFrame = function () {
    frame++;
    if (frame == 2) {
        // All of them have been rendered (or skipped) at their positions now.
        b._x = 200;
        show("rendered at x=0, then _x=200", b);
        c._xscale = 300;
        show("rendered at x=0, then _xscale=300", c);
        d._rotation = 180;
        show("rendered at x=0, then _rotation=180", d);
        e._x = 200;
        e._x = 0;
        show("rendered at x=0, then _x=200 and back to 0", e);
        p._x = 200;
        show("parent moved to x=200: child", child);
        off._x = 0;
        show("was at x=-1000 (off stage), then _x=0", off);
        inv._x = 200;
        inv._visible = true;
        show("invisible at x=0, then _x=200 and made visible", inv);
        inv2._visible = false;
        show("rendered at x=0, then made invisible", inv2);
        inv2._x = 200;
        show("rendered at x=0, then made invisible and _x=200", inv2);
        big._xscale = 300;
        big._x = -100;
        show("rendered at x=0, then _xscale=300 _x=-100", big);
        bd._xscale = 300;
        bmp.draw(bd);
        show("rendered at x=0, then _xscale=300 and BitmapData.draw", bd);
    }
    if (frame == 3) {
        show("next frame: b", b);
        show("next frame: inv2 (invisible since last frame)", inv2);
        inv2._visible = true;
        show("next frame: inv2 made visible", inv2);
        delete onEnterFrame;
    }
};

// Tests setting _width and _height of clips without a width, a height or both,
// unrotated and rotated. Each clip gets a new _width, then a new _height, then
// a negative _width, and traces its size and scale after each step. The
// loading bars at the end grow from a width of 0, and the text field at the
// end gets negative and zero sizes.

function make(name, kind) {
    var mc = _root.createEmptyMovieClip(name, _root.getNextHighestDepth());

    if (kind == "hline") {
        mc.beginFill(0xFF0000);
        mc.moveTo(0, 0);
        mc.lineTo(100, 0);
        mc.lineTo(0, 0);
        mc.endFill();
    } else if (kind == "vline") {
        mc.beginFill(0xFF0000);
        mc.moveTo(0, 0);
        mc.lineTo(0, 100);
        mc.lineTo(0, 0);
        mc.endFill();
    }

    return mc;
}

function show(label, mc) {
    trace(label + ": _width " + mc._width + ", _height " + mc._height + ", _xscale " + mc._xscale + ", _yscale " + mc._yscale);
}

var kinds = ["hline", "vline", "empty"];

for (var k = 0; k < kinds.length; k++) {
    for (var r = 0; r <= 30; r += 30) {
        var mc = make(kinds[k] + r, kinds[k]);
        var label = kinds[k] + " rot " + r;
        mc._rotation = r;
        show(label + " initial", mc);

        mc._width = 200;
        show(label + " _width = 200", mc);

        mc._height = 50;
        show(label + " _height = 50", mc);

        mc._width = -10;
        show(label + " _width = -10", mc);
    }
}

// A loading bar that is grown from a width of 0, as in Flash 3DX.
var bar = make("bar", "hline");
bar._xscale = 0;
show("bar _xscale = 0", bar);

for (var i = 0; i < 3; i++) {
    bar._width += 1;
    show("bar _width += 1", bar);
}

var bar2 = make("bar2", "vline");

for (var i = 0; i < 2; i++) {
    bar2._width += 1;
    show("vline bar _width += 1", bar2);
}

_root.createTextField("tf", _root.getNextHighestDepth(), 0, 0, 100, 50);
var tf = _root.tf;
var sizes = [-10, 0];

for (var i = 0; i < sizes.length; i++) {
    tf._width = sizes[i];
    trace("text field _width = " + sizes[i] + ": _width " + tf._width + ", _height " + tf._height);

    tf._height = sizes[i];
    trace("text field _height = " + sizes[i] + ": _width " + tf._width + ", _height " + tf._height);
}

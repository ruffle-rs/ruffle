// Tests setting _width and _height of clips that are scaled, rotated or both.
//
// For every combination of size, _xscale and _rotation below, one new clip
// gets a new _width and another one a new _height. The edge cases at the end
// cover negative _yscale and other combinations. Each line traces _width,
// _height, _xscale and _yscale before and after the assignment.

function make(w, h) {
    var depth = _root.getNextHighestDepth();
    var mc = _root.createEmptyMovieClip("c" + depth, depth);
    mc.beginFill(0xFF0000);
    mc.moveTo(0, 0);
    mc.lineTo(w, 0);
    mc.lineTo(w, h);
    mc.lineTo(0, h);
    mc.endFill();

    return mc;
}

function describe(mc) {
    return "_width " + mc._width + ", _height " + mc._height + ", _xscale " + mc._xscale + ", _yscale " + mc._yscale;
}

function test(w, h, xscale, yscale, rotation, property, value) {
    var mc = make(w, h);
    mc._xscale = xscale;
    mc._yscale = yscale;
    mc._rotation = rotation;

    var label = w + "x" + h + " scale " + xscale + "/" + yscale + " rot " + rotation;
    var before = describe(mc);
    mc[property] = value;
    trace(label + " " + property + " = " + value + ": " + before + " -> " + describe(mc));
}

var sizes = [0, 50, 100];
var scales = [-100, -50, 0, 50, 200];
var rotations = [0, 10, 45, 90];

for (var wi = 0; wi < sizes.length; wi++) {
    for (var hi = 0; hi < sizes.length; hi++) {
        for (var si = 0; si < scales.length; si++) {
            for (var ri = 0; ri < rotations.length; ri++) {
                var w = sizes[wi];
                var h = sizes[hi];
                var xscale = scales[si];
                var rotation = rotations[ri];

                test(w, h, xscale, 100, rotation, "_width", 150);
                test(w, h, xscale, 100, rotation, "_height", 80);
            }
        }
    }
}

var edgeCases = [
    // [  w,  h, xscale, yscale, rotation,  property, value]
    [100, 50,    100,    -50,        0,  "_width",   150],
    [100, 50,    100,    -50,       30, "_height",    80],
    [100, 50,    -50,   -100,       45,  "_width",   150],
    [100, 50,    200,     50,       30, "_height",   100],
    [100, 50,    100,    100,      170,  "_width",   150],
    [100, 50,    100,    100,     -135, "_height",    80],
    [100, 50,    100,    100,       30,  "_width",     0],
    [100, 50,    100,    100,       30, "_height",   -10]
];

for (var i = 0; i < edgeCases.length; i++) {
    var c = edgeCases[i];
    var w = c[0];
    var h = c[1];
    var xscale = c[2];
    var yscale = c[3];
    var rotation = c[4];
    var property = c[5];
    var value = c[6];

    test(w, h, xscale, yscale, rotation, property, value);
}

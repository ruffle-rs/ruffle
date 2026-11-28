// Tests setting _width and _height of rotated clips with several children to
// their current values, and traces the size and scale before and after. The
// clips have two diagonal squares, a rotated nested child, a text field
// child, a scroll rect, or a flip. The marbles in Chinese Checkers set their
// size like this after every move of the computer player, at rotations of
// -59, 59, -121 or 121 depending on the player's colour.

function square(parent, name, depth, x, y, size) {
    var mc = parent.createEmptyMovieClip(name, depth);
    mc.beginFill(0xFF0000);
    mc.moveTo(0, 0);
    mc.lineTo(size, 0);
    mc.lineTo(size, size);
    mc.lineTo(0, size);
    mc.endFill();
    mc._x = x;
    mc._y = y;

    return mc;
}

function show(label, mc) {
    trace(label + ": _width " + mc._width + ", _height " + mc._height + ", _xscale " + mc._xscale + ", _yscale " + mc._yscale);
}

var rotations = [0, 45, 59, -59, 121, -121, 180];

for (var i = 0; i < rotations.length; i++) {
    var rotation = rotations[i];

    var diagonal = _root.createEmptyMovieClip("diagonal" + i, i * 3 + 1);
    square(diagonal, "a", 1, 0, 0, 20);
    square(diagonal, "b", 2, 20, 20, 20);
    diagonal._rotation = rotation;

    var label = "diagonal rot " + rotation;
    show(label + " before", diagonal);

    diagonal._width = diagonal._width;
    diagonal._height = diagonal._height;
    show(label + " size reset", diagonal);

    var nested = _root.createEmptyMovieClip("nested" + i, i * 3 + 2);
    var inner = nested.createEmptyMovieClip("inner", 1);
    square(inner, "a", 1, -15, -15, 30);
    inner._rotation = 45;
    square(nested, "b", 2, -10, -10, 20);
    nested._rotation = rotation;

    label = "nested rot " + rotation;
    show(label + " before", nested);

    nested._width = nested._width;
    nested._height = nested._height;
    show(label + " size reset", nested);

    var single = _root.createEmptyMovieClip("single" + i, i * 3 + 3);
    square(single, "a", 1, 0, 0, 30);
    single._rotation = rotation;

    label = "single rot " + rotation;
    show(label + " before", single);

    single._width = 60;
    show(label + " _width = 60", single);
}

var flips = [[-100, 100, 45], [100, -100, 45], [-100, 100, 121], [-100, -100, 59]];

for (var i = 0; i < flips.length; i++) {
    var f = flips[i];
    var xscale = f[0];
    var yscale = f[1];
    var rotation = f[2];

    var flipped = _root.createEmptyMovieClip("flipped" + i, 100 + i);
    square(flipped, "a", 1, 0, 0, 20);
    square(flipped, "b", 2, 20, 20, 20);
    flipped._xscale = xscale;
    flipped._yscale = yscale;
    flipped._rotation = rotation;

    var label = "diagonal scale " + xscale + "/" + yscale + " rot " + rotation;
    show(label + " before", flipped);

    flipped._width = flipped._width;
    flipped._height = flipped._height;
    show(label + " size reset", flipped);
}

var rotations2 = [0, 59, -121];

for (var i = 0; i < rotations2.length; i++) {
    var rotation = rotations2[i];

    var withText = _root.createEmptyMovieClip("withText" + i, 200 + i);
    square(withText, "a", 1, 0, 0, 20);
    withText.createTextField("text", 2, 20, 20, 40, 20);
    withText._rotation = rotation;

    var label = "text child rot " + rotation;
    show(label + " before", withText);

    withText._width = withText._width;
    withText._height = withText._height;
    show(label + " size reset", withText);

    var scrolled = _root.createEmptyMovieClip("scrolled" + i, 300 + i);
    square(scrolled, "a", 1, 0, 0, 20);
    square(scrolled, "b", 2, 20, 20, 20);
    scrolled.scrollRect = new flash.geom.Rectangle(5, 5, 30, 20);
    scrolled._rotation = rotation;

    label = "scroll rect rot " + rotation;
    show(label + " before", scrolled);

    scrolled._width = scrolled._width;
    scrolled._height = scrolled._height;
    show(label + " size reset", scrolled);
}

trace("Done");

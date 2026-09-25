var map = _root.createEmptyMovieClip("map", 1);

map.beginFill(0xFF0000, 100);
map.moveTo(0, 0);
map.lineTo(10, 0);
map.lineTo(10, 10);
map.lineTo(0, 10);
map.lineTo(0, 0);
map.endFill();

function testDraw(label, source)
{
    var dest = new flash.display.BitmapData(
            10,
            10,
            true,
            0x00000000
        );

    var result = dest.draw(source);
    var pixel = dest.getPixel32(5, 5);

    trace(
            label +
            ": result=" + result +
            ", pixel=" + pixel.toString(16)
        );
}

trace("MovieClip object");
testDraw("map object", map);

trace("String targets");

var paths = [
        "map",
        "/map",
        "_root.map",
        "_root/map",
        "_root:map",
        "_level0.map",
        "_level0/map",
        "_level0:map",
        ":map",
        "/:map",
        ".map",
        "/.map",

        "map/",
        "/map/",
        "map/..",
        "map/../map",
        "map/..:map",

        "",
        "/",
        "//",
        ".",
        "..",

        "missing",
        "/missing",
        "_root.missing",
        "_root/missing",
        "_root:missing",
        "_level0.missing",
        "_level0/missing",
        "_level0:missing",
        ":missing",
        "/:missing"
    ];

for (var i = 0; i < paths.length; i++)
{
    var path = paths[i];
    testDraw('"' + path + '"', path);
}

trace("Non-string targets");

var o = {};
o.toString = function () {
    trace("toString called");
    return "map";
};
o.valueOf = function () {
    trace("valueOf called");
    return "map";
};
testDraw("object coerced to string", o);

_root.createEmptyMovieClip("1234", 2);
testDraw("number", 1234);
_root.createEmptyMovieClip("true", 3);
testDraw("true", true);
_root.createEmptyMovieClip("Infinity", 4);
testDraw("Infinity", 1.0/0.0);

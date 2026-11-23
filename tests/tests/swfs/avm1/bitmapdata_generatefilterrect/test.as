// Calls BitmapData.generateFilterRect with different kinds of filters, with
// source rectangles that are fractional, partly or fully outside the bitmap,
// and with invalid arguments.

var bitmap = new flash.display.BitmapData(100, 80, true, 0);
var source = new flash.geom.Rectangle(10, 10, 25, 25);
var blur = new flash.filters.BlurFilter(4, 4, 1);

function show(label, sourceRect, filter) {
    trace(label + ": " + bitmap.generateFilterRect(sourceRect, filter));
}

trace("// Filters");
show("blur 4 quality 1", source, blur);
show("blur 4 quality 3", source, new flash.filters.BlurFilter(4, 4, 3));
show("blur 0 quality 1", source, new flash.filters.BlurFilter(0, 0, 1));
show("glow 6 quality 2", source, new flash.filters.GlowFilter(0xFF0000, 1, 6, 6, 2, 2));
show("convolution 4x6", source, new flash.filters.ConvolutionFilter(4, 6, []));
show("color matrix", source, new flash.filters.ColorMatrixFilter());
show("drop shadow", source, new flash.filters.DropShadowFilter(10, 135, 0, 1, 4, 4, 1, 2));
show("gradient glow", source, new flash.filters.GradientGlowFilter(10, 135, [0, 0xFF0000], [0, 1], [0, 255], 4, 4, 1, 2));
show("bevel", source, new flash.filters.BevelFilter(10, 135, 0xFFFFFF, 1, 0, 1, 4, 4, 1, 2));
show("gradient bevel", source, new flash.filters.GradientBevelFilter(10, 135, [0, 0xFF0000], [0, 1], [0, 255], 4, 4, 1, 2));
show("drop shadow, distance -1000.5", source, new flash.filters.DropShadowFilter(-1000.5, 0, 0, 1, 0, 0, 1, 1));

var corner = new flash.geom.Rectangle(90, 70, 10, 10);
show("glow", corner, new flash.filters.GlowFilter(0, 1, 20, 20, 1, 1, false));
show("inner glow", corner, new flash.filters.GlowFilter(0, 1, 20, 20, 1, 1, true));
show("outer bevel", corner, new flash.filters.BevelFilter(30, 45, 0xFFFFFF, 1, 0, 1, 20, 20, 1, 1, "outer"));

var map = new flash.display.BitmapData(4, 4, false, 0xFF0000);
var middle = new flash.geom.Rectangle(40, 30, 20, 20);
var origin = new flash.geom.Point(0, 0);
show("displacement map", middle, new flash.filters.DisplacementMapFilter(map, origin, 1, 1, 100, 50, "wrap"));
show("displacement map, color mode", middle, new flash.filters.DisplacementMapFilter(map, origin, 1, 1, 100, 50, "color"));
show("displacement map, invalid component", middle, new flash.filters.DisplacementMapFilter(map, origin, 3, 3, 50, 30, "color"));
show("displacement map, component 257", middle, new flash.filters.DisplacementMapFilter(map, origin, 257, 257, 50, 30, "color"));

trace("// Source rectangles");
show("fractional", new flash.geom.Rectangle(10.5, 11.5, 20.5, 20.7), blur);
show("over the right and bottom edges", new flash.geom.Rectangle(90, 70, 20, 20), blur);
show("outside", new flash.geom.Rectangle(200, 200, 10, 10), blur);
show("plain object", {x: 10, y: 10, width: 25, height: 25}, blur);
show("NaN", new flash.geom.Rectangle(NaN, 10, 20, 10), blur);
show("string", new flash.geom.Rectangle("abc", 10, 20, 10), blur);
show("undefined height", {x: 1, y: 2, width: 3, height: undefined}, blur);
show("out of range", new flash.geom.Rectangle(4294967306, 10, 20, 10), blur);
show("maximum x", new flash.geom.Rectangle(2147483647, 10, 20, 10), blur);
show("maximum width", new flash.geom.Rectangle(10, 10, 2147483647, 10), blur);
show("negative fractional width", new flash.geom.Rectangle(10, 10, -0.4, 10), blur);

var colorMatrix = new flash.filters.ColorMatrixFilter();
var edges = [
    [-10, -1],
    [-1, 0.5],
    [0, -0.5],
    [NaN, 2147483660],
    [-2147483648, 2147483700],
    [-10, 2147483658]
];

for (var i = 0; i < edges.length; i++) {
    var x = edges[i][0];
    var width = edges[i][1];
    show("x " + x + ", width " + width, {x: x, y: 10, width: width, height: 20}, colorMatrix);
    show("y " + x + ", height " + width, {x: 10, y: x, width: 20, height: width}, colorMatrix);
}

var inherited = {};
inherited.__proto__ = {x: 5, y: 5, width: 10, height: 10};
show("inherited properties", inherited, blur);

var result = bitmap.generateFilterRect(source, blur);
trace("result is a Rectangle: " + (result.__proto__ == flash.geom.Rectangle.prototype));

trace("// Invalid arguments");
show("no source rectangle", undefined, blur);
show("no filter", source, undefined);
show("plain object as filter", source, {blurX: 4, blurY: 4, quality: 1});
trace("no arguments: " + bitmap.generateFilterRect());
trace("only a source rectangle: " + bitmap.generateFilterRect(source));
bitmap.dispose();
show("disposed bitmap", source, blur);

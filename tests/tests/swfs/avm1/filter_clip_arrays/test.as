// Tests filter constructors and properties that take an array or a point
// with movie clips in their place: ColorMatrixFilter.matrix, the colors,
// alphas and ratios of GradientGlowFilter, and
// DisplacementMapFilter.mapPoint. Each clip has `length` and indexed
// elements, or `x` and `y`. Each value is passed to the constructor and,
// on a filter created with other values, assigned to the property.

function makeArrayClip(name, values) {
    var clip = createEmptyMovieClip(name, getNextHighestDepth());
    clip.length = values.length;

    for (var i = 0; i < values.length; i++) {
        clip[i] = values[i];
    }

    return clip;
}

var matrixValues = [];

for (var i = 0; i < 20; i++) {
    matrixValues.push(i + 1);
}

var matrixClip = makeArrayClip("matrixClip", matrixValues);
var colorsClip = makeArrayClip("colorsClip", [0xFF0000, 0x00FF00]);
var alphasClip = makeArrayClip("alphasClip", [0.25, 0.5]);
var ratiosClip = makeArrayClip("ratiosClip", [10, 200]);

var pointClip = createEmptyMovieClip("pointClip", getNextHighestDepth());
pointClip.x = 10;
pointClip.y = 20;

trace("// ColorMatrixFilter");
var colorMatrix = new flash.filters.ColorMatrixFilter(matrixClip);
trace("constructor: " + colorMatrix.matrix);
colorMatrix = new flash.filters.ColorMatrixFilter();
colorMatrix.matrix = matrixClip;
trace("property: " + colorMatrix.matrix);
trace("");

trace("// GradientGlowFilter");
var glow = new flash.filters.GradientGlowFilter(4, 45, colorsClip, alphasClip, ratiosClip);
trace("constructor: colors: " + glow.colors + ", alphas: " + glow.alphas + ", ratios: " + glow.ratios);
glow = new flash.filters.GradientGlowFilter(4, 45, [1, 2], [1, 1], [0, 255]);
glow.alphas = alphasClip;
glow.ratios = ratiosClip;
trace("alphas and ratios properties: alphas: " + glow.alphas + ", ratios: " + glow.ratios);
glow.colors = colorsClip;
trace("colors property: colors: " + glow.colors + ", alphas: " + glow.alphas + ", ratios: " + glow.ratios);
trace("");

trace("// DisplacementMapFilter");
var mapBitmap = new flash.display.BitmapData(2, 2, true, 0);
var displacement = new flash.filters.DisplacementMapFilter(mapBitmap, pointClip, 1, 2, 3, 4);
trace("constructor: " + displacement.mapPoint);
displacement = new flash.filters.DisplacementMapFilter(mapBitmap, new flash.geom.Point(1, 2), 1, 2, 3, 4);
displacement.mapPoint = pointClip;
trace("property: " + displacement.mapPoint);

function draw(x1, y1, x2, y2, callback) {
	var depth = _root.getNextHighestDepth();
	var mc = _root.createEmptyMovieClip("mc_" + depth, depth);
	mc.lineStyle(8);
	callback(mc);
	mc.moveTo(x1, y1);
	mc.lineTo(x1, y2);
	mc.lineTo(x2, y2);
	mc.lineTo(x2, y1);
	mc.lineTo(x1, y1);
	mc.endFill();
}
var colors = [0xFF1419, 0x0E14FF];
var matrix = {matrixType:"box", x:100, y:100, w:200, h:200, r:(45 / 180) * Math.PI};


draw(10,  10, 100, 100, function(mc) { mc.beginGradientFill("linear", colors, [100, 100], [0, 255],   matrix); });
draw(110, 10, 200, 100, function(mc) { mc.beginGradientFill("linear", colors, [50, 50],   [100, 100], matrix, "pad"); });
draw(210, 10, 300, 100, function(mc) { mc.beginGradientFill("linear", colors, [80, 40],   [100, 200], matrix, "pad", "RGB"); });
draw(310, 10, 400, 100, function(mc) { mc.beginGradientFill("linear", colors, [0, 100],   [0, 255],   matrix, "pad", "RGB", 0); });
draw(410, 10, 500, 100, function(mc) { mc.beginGradientFill("linear", colors, [100, 100], [0, 255],   matrix, "???", "???", "???"); });

draw(10,  110, 100, 200, function(mc) { mc.beginGradientFill("linear", colors, [100, 100], [0, 255.9],   matrix, "reflect", "linearRGB"); });
draw(110, 110, 200, 200, function(mc) { mc.beginGradientFill("linear", colors, [100, 80],  [0, 255],     matrix, "repeat",  "linearRGB", 88); });
draw(210, 110, 300, 200, function(mc) { mc.beginGradientFill("linear", colors, [77, 55],   [NaN, 200],   matrix, "reflect", "linearRGB", -1000); });
draw(310, 110, 400, 200, function(mc) { mc.beginGradientFill("radial", colors, [100, 100], [-0.99, 255], matrix); });
draw(410, 110, 500, 200, function(mc) { mc.beginGradientFill("radial", colors, [100, 80],  [0, 255],     matrix, "repeat",  "linearRGB", 0.8); });

draw(10,  210, 100, 300, function(mc) { mc.beginGradientFill("radial", colors, [100, 100], [50, 50], matrix, "reflect", "linearRGB"); });
draw(110, 210, 200, 300, function(mc) { mc.beginGradientFill("radial", colors, [-10, 300], [0, 255], matrix, "???????", "???", "???"); });
draw(210, 210, 300, 300, function(mc) { mc.beginGradientFill("radial", colors, [100, 100], [0, 255], matrix); mc.beginGradientFill(null, 1, 2, 3, 4, 5, 6, 7, 8, 9); });
draw(310, 210, 400, 300, function(mc) { mc.beginGradientFill("radial", colors, [100, 100], [0, 255], matrix); mc.beginGradientFill(undefined, 1, 2, 3, 4, 5, 6, 7, 8, 9); });


// Invalid cases (won't draw anything but a black border)
draw(410, 210, 420, 300, function(mc) { mc.beginGradientFill("linear", colors, [100, 100], [0, 255]); }); // too few arguments
draw(420, 210, 430, 300, function(mc) { mc.beginGradientFill("linear", colors, [100, 100], [0, 255],  matrix, "pad", "RGB", 0, "invalid"); }); // too many arguments
draw(430, 210, 440, 300, function(mc) { mc.beginGradientFill("LINEAR", colors, [100, 100], [0, 255],  matrix); }); // invalid fill type
draw(440, 210, 450, 300, function(mc) { mc.beginGradientFill("linear", colors, [100, 100], [255],     matrix); }); // number of items in the colors, alphas and ratios arrays are not equal
draw(450, 210, 460, 300, function(mc) { mc.beginGradientFill("linear", colors, [100, 100], [-1, 255], matrix); }); // invalid ratio (ratios[0] <= -1.0)
draw(460, 210, 470, 300, function(mc) { mc.beginGradientFill("linear", colors, [100, 100], [0, 256],  matrix); }); // invalid ratio (ratios[1] >= 256.0)

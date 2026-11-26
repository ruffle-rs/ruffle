// Tests the default value of `mergeAlpha` in `BitmapData.copyPixels`
// when an `alphaBitmap` is supplied but `mergeAlpha` is omitted.
// Flash Player treats the omitted argument as `false`.

Rect = flash.geom.Rectangle;
Point = flash.geom.Point;
BitmapData = flash.display.BitmapData;

function makeAlphaBitmap() {
    var alpha = new BitmapData(2, 1, true, 0);
    alpha.setPixel32(0, 0, 0x00000000);
    alpha.setPixel32(1, 0, 0xFF000000);
    return alpha;
}

function dumpAlpha(bmp) {
    var s = "";
    for (var x = 0; x < bmp.width; ++x) {
        if (x > 0) {
            s += " / ";
        }
        s += (bmp.getPixel32(x, 0) >>> 24);
    }
    trace(s);
}

function runTest(label, selfCopy, mode) {
    var dest = new BitmapData(2, 1, true, 0xFF102030);
    var src = selfCopy ? dest : new BitmapData(2, 1, true, 0xFF405060);
    var alpha = makeAlphaBitmap();
    var rect = new Rect(0, 0, 2, 1);
    var pt = new Point(0, 0);
    var alphaPt = new Point(0, 0);

    if (mode == "omitted") {
        dest.copyPixels(src, rect, pt, alpha, alphaPt);
    } else if (mode == "false") {
        dest.copyPixels(src, rect, pt, alpha, alphaPt, false);
    } else {
        dest.copyPixels(src, rect, pt, alpha, alphaPt, true);
    }

    trace(label + ", mergeAlpha " + mode + ":");
    dumpAlpha(dest);
}

runTest("distinct", false, "omitted");
runTest("distinct", false, "false");
runTest("distinct", false, "true");
runTest("self", true, "omitted");
runTest("self", true, "false");
runTest("self", true, "true");

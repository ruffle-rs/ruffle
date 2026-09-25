Rect = flash.geom.Rectangle;
Point = flash.geom.Point;

function dump(bmp) {
    var w = bmp.width;
    var h = bmp.height;
    var transparent = bmp.transparent;
    for (var y = 0; y < h; ++y) {
        var s = "";
        for (var x = 0; x < w; ++x) {
            var v = transparent ? bmp.getPixel32(x, y) : bmp.getPixel(x, y);
            s += " " + v.toString(16);
        }
        trace(s);
    }
}

function runTest(label, destPoint, mergeAlpha, transparent) {
    trace(label);
    var bmp = new flash.display.BitmapData(8, 8, transparent, 0x7f000000);
    for (var j = 0; j < 8; ++j) {
        for (var i = 0; i < 8; ++i) {
            bmp.setPixel(i, j, (j + 1) * 16 + (i + 1));
        }
    }
    bmp.copyPixels(bmp, new Rect(2, 2, 4, 4), destPoint, null, null, mergeAlpha);
    dump(bmp);
}

function runTestSuite(prefix, mergeAlpha, transparent) {
    runTest(prefix + ", 0",       new Point(2, 2), mergeAlpha, transparent);
    runTest(prefix + ", dx+ dy+", new Point(3, 3), mergeAlpha, transparent);
    runTest(prefix + ", dx- dy-", new Point(1, 1), mergeAlpha, transparent);
    runTest(prefix + ", dx+ dy-", new Point(3, 1), mergeAlpha, transparent);
    runTest(prefix + ", dx- dy+", new Point(1, 3), mergeAlpha, transparent);
    runTest(prefix + ", dx0 dy+", new Point(2, 3), mergeAlpha, transparent);
    runTest(prefix + ", dx0 dy-", new Point(2, 1), mergeAlpha, transparent);
    runTest(prefix + ", dx+ dy0", new Point(3, 2), mergeAlpha, transparent);
    runTest(prefix + ", dx- dy0", new Point(1, 2), mergeAlpha, transparent);

    runTest(prefix + ", dx++ dy++", new Point(6, 6), mergeAlpha, transparent);
    runTest(prefix + ", dx++ dy0", new Point(6, 2), mergeAlpha, transparent);
    runTest(prefix + ", dx0 dy++", new Point(2, 6), mergeAlpha, transparent);

    runTest(prefix + ", dx-- dy--", new Point(-2, -2), mergeAlpha, transparent);
    runTest(prefix + ", dx-- dy0", new Point(-2, 2), mergeAlpha, transparent);
    runTest(prefix + ", dx0 dy--", new Point(2, -2), mergeAlpha, transparent);

    runTest(prefix + ", dx++ dy--", new Point(6, -2), mergeAlpha, transparent);
    runTest(prefix + ", dx-- dy++", new Point(-2, 6), mergeAlpha, transparent);
}

runTestSuite("-mergeAlpha, -transparent", false, false);
runTestSuite("+mergeAlpha, -transparent", true, false);
runTestSuite("-mergeAlpha, +transparent", false, true);
runTestSuite("+mergeAlpha, +transparent", true, true);

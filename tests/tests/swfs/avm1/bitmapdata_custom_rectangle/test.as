import flash.display.BitmapData;

trace("===== before");

var bd = new BitmapData(3, 3, true, 0xAABBCCDD);
var sourceRect = bd.rectangle;

var noisyMask = {
    valueOf: function() {
        trace("mask.valueOf called!");
        return 0xFF000000;
    }
};

var noisyColor = {
    valueOf: function() {
        trace("color.valueOf called!");
        return 0xAA000000;
    }
};

var filter = new flash.filters.BevelFilter();
trace("rectangle: " + bd.rectangle);
trace("generateFilterRect: " + bd.generateFilterRect(sourceRect, filter));
trace("getColorBoundsRect: " + bd.getColorBoundsRect(noisyMask, noisyColor));

trace("===== noisy function");
flash.geom.Rectangle = function(a,b,c,d,e) {
    trace("Rectangle:");
    trace("  a=" + a);
    trace("  b=" + b);
    trace("  c=" + c);
    trace("  d=" + d);
    trace("  e=" + e);
};

trace("rectangle: " + bd.rectangle);
trace("generateFilterRect: " + bd.generateFilterRect(sourceRect, filter));
trace("getColorBoundsRect: " + bd.getColorBoundsRect(noisyMask, noisyColor));

trace("===== throwing function");
flash.geom.Rectangle = function() {
    throw "test";
};

try {
    trace("rectangle: " + bd.rectangle);
} catch (e) {
    trace("Caught: " + e);
}

try {
    trace("generateFilterRect: " + bd.generateFilterRect(sourceRect, filter));
} catch (e) {
    trace("Caught: " + e);
}

try {
    trace("getColorBoundsRect: " + bd.getColorBoundsRect(noisyMask, noisyColor));
} catch (e) {
    trace("Caught: " + e);
}

trace("===== not an object");
flash.geom = 4;

trace("rectangle: " + bd.rectangle);
trace("generateFilterRect: " + bd.generateFilterRect(sourceRect, filter));
trace("getColorBoundsRect: " + bd.getColorBoundsRect(noisyMask, noisyColor));

var opaque = new flash.display.BitmapData(1, 1);
var transparent = new flash.display.BitmapData(1, 1, true, 0);
var faint = new flash.display.BitmapData(1, 1, true, 0x01000000);
var origin = new flash.geom.Point(0, 0);
trace(opaque.hitTest(origin, 255, transparent, origin));
trace(opaque.hitTest(origin, 255, transparent));
trace(opaque.hitTest(origin, 255, transparent, origin, undefined));
trace(opaque.hitTest(origin, 255, faint, origin));

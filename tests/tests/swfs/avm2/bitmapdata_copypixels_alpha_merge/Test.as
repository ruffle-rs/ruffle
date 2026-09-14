package {
    import flash.display.Sprite;
    import flash.display.BitmapData;
    import flash.geom.Rectangle;
    import flash.geom.Point;

    // copyPixels with alphaBitmapData across source/alpha transparency and
    // mergeAlpha combinations. Ruffle still diverges from FP on some of these
    // (premultiply rounding in the merge path).
    public class Test extends Sprite {
        public function Test() {
            var rect:Rectangle = new Rectangle(0, 0, 1, 1);
            var origin:Point = new Point(0, 0);
            var sas:Array = [255, 254, 128, 3];
            var aas:Array = [255, 254, 128, 1, 0];
            // Full-pixel spot checks across transparency and merge combinations
            for each (var st:Boolean in [true, false]) {
                for each (var at:Boolean in [true, false]) {
                    for each (var merge:Boolean in [true, false]) {
                        var line2:String = "st=" + st + " at=" + at + " m=" + merge + ":";
                        for each (var s2:int in sas) {
                            for each (var a2:int in aas) {
                                var src2:BitmapData = new BitmapData(1, 1, st, ((s2 << 24) | 0x00FF00) >>> 0);
                                var alp2:BitmapData = new BitmapData(1, 1, at, (a2 << 24) >>> 0);
                                var dst2:BitmapData = new BitmapData(1, 1, true, 0x40332211);
                                dst2.copyPixels(src2, rect, origin, alp2, origin, merge);
                                line2 += " " + dst2.getPixel32(0, 0).toString(16);
                            }
                        }
                        trace(line2);
                    }
                }
            }

            trace("done");
        }
    }
}

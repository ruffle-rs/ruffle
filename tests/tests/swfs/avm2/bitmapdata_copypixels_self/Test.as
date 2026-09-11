package {
    import flash.display.Sprite;
    import flash.display.BitmapData;
    import flash.geom.Rectangle;
    import flash.geom.Point;

    public class Test extends Sprite {
        public function Test() {
            runTestSuite("-mergeAlpha, -transparent", false, false);
            runTestSuite("+mergeAlpha, -transparent", true, false);
            runTestSuite("-mergeAlpha, +transparent", false, true);
            runTestSuite("+mergeAlpha, +transparent", true, true);
        }

        private function dump(bmp:BitmapData):void {
            var w:int = bmp.width;
            var h:int = bmp.height;
            var transparent:Boolean = bmp.transparent;
            for (var y:int = 0; y < h; ++y) {
                var s:String = "";
                for (var x:int = 0; x < w; ++x) {
                    var v:uint = transparent ? bmp.getPixel32(x, y) : bmp.getPixel(x, y);
                    s += " " + v.toString(16);
                }
                trace(s);
            }
        }

        private function runTest(label:String, destPoint:Point, mergeAlpha:Boolean, transparent:Boolean):void {
            trace(label);
            var bmp:BitmapData = new BitmapData(8, 8, transparent, 0x7f000000);
            for (var j:int = 0; j < 8; ++j) {
                for (var i:int = 0; i < 8; ++i) {
                    bmp.setPixel(i, j, (j + 1) * 16 + (i + 1));
                }
            }
            bmp.copyPixels(bmp, new Rectangle(2, 2, 4, 4), destPoint, null, null, mergeAlpha);
            dump(bmp);
        }

        private function runTestSuite(prefix:String, mergeAlpha:Boolean, transparent:Boolean):void {
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
    }
}

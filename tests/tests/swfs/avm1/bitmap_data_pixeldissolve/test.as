
import flash.display.BitmapData;
import flash.geom.Point;
import flash.geom.Rectangle;
import flash.utils.Timer;
import flash.events.TimerEvent;

class test {

    private function countTraceBitmap(bitmap, round, fillColor) {

       var count = 0;

       var x = 0;
       while (x < bitmap.width)
       {
          var y = 0;
          while (y < bitmap.height)
          {
             if (bitmap.getPixel32(x, y) == fillColor) {
                count += 1;
             }

             y += 1;
          }

          x += 1;
       }

       trace("(" + round + ") Overwritten pixel count: " + count);
    }

    private function dissolveOnce(bitmap, rectangle, destPoint, rand: Number, numPixels: Number): Number {
        var red = 0xFFFF0000;
        var newRand: Number = bitmap.pixelDissolve(bitmap, rectangle, destPoint, rand, numPixels, red);
        return newRand;
    }

    private function dissolveRounds(
        rounds,
        width, height,
        useBitmapDataOwnRectangle: Boolean,
        sourceRect,
        destPoint,
        numPixels
    ) {

        rounds = Math.max(1, rounds);

        var sourceRectString: String = useBitmapDataOwnRectangle ? "sourceRect: [own]" : "sourceRect: " + sourceRect;

        trace("Dissolving for " + rounds + " rounds, with data: width: " + width + ", height: " + height + ", " + sourceRectString + ", destPoint: " + destPoint + ", numPixels: " + numPixels);

        var bmd2 = new flash.display.BitmapData(width, height, false, 0xFFCCCCCC);

        var round;
        var randNum: Number = 0;
        for (round = 1; round <= rounds; round++)
        {
            randNum = dissolveOnce(bmd2, useBitmapDataOwnRectangle ? bmd2.rectangle : sourceRect, destPoint, randNum, numPixels);
            countTraceBitmap(bmd2, round, 0xFFFF0000);

            if (randNum < 0) {
                trace("Result: " + randNum);
            }
        }

        trace("");
    }

    private function dissolveRounds_1(
        rounds,
        width, height,
        numPixels
    ) {
        dissolveRounds(rounds, width, height, true, null, new Point(0, 0), numPixels);
    }

    private function dissolveRounds_2(
        rounds,
        width, height,
        sourceRect,
        numPixels
    ) {
        dissolveRounds(rounds, width, height, false, sourceRect, new Point(0, 0), numPixels);
    }

    private function dissolveRounds_3(
        rounds,
        width, height,
        sourceRect,
        destPoint,
        numPixels
    ) {
        dissolveRounds(rounds, width, height, false, sourceRect, destPoint, numPixels);
    }

    public function test() {

        trace("------------------------------------------");
        trace("Basic test.");
        trace("");
        dissolveRounds_1(100, 10, 10, 1);

        trace("");
        trace("");
        trace("------------------------------------------");
        trace("`numPixels`.");
        trace("");
        dissolveRounds_1(4, 10, 10, 0);
        // This apparently bugs out at least one version of Flash Player on AVM1.
        //try {
        //    dissolveRounds_1(4, 10, 10, -1);
        //}
        //catch (e:Error) {
        //    trace("Negative `numPixels` should error.");
        //}
        dissolveRounds_1(35, 10, 10, 3);

        trace("");
        trace("");
        trace("------------------------------------------");
        trace("Dimensions.");
        trace("");
        // WARNING: Apparently, `width` or `height` being 1 means that nothing is written,
        // not even the pixel at (0, 0). This seems like a bug in Flash Player and will
        // not necessarily be emulated. So, do not test for it here.
        dissolveRounds_1(61, 150, 2, 5);
        dissolveRounds_1(61, 2, 150, 5);
        dissolveRounds_1(66, 8, 8, 1);
        dissolveRounds_1(66, 7, 9, 1); // 7*9 = 8*8 - 1.
        dissolveRounds_1(66, 5, 13, 1); // 5*13 = 8*8 + 1.

        trace("");
        trace("");
        trace("------------------------------------------");
        trace("`sourceRect`.");
        trace("");
        dissolveRounds_2(101, 10, 10, new Rectangle(0, 0, 10, 10), 1);
        dissolveRounds_2(20, 10, 10, new Rectangle(0, 0, 4, 4), 1);
        dissolveRounds_2(35, 10, 10, new Rectangle(0, 0, 8, 4), 1);
        dissolveRounds_2(35, 10, 10, new Rectangle(0, 0, 4, 8), 1);
        dissolveRounds_2(27, 10, 10, new Rectangle(0, 0, 3, 8), 1);
        dissolveRounds_2(31, 10, 10, new Rectangle(0, 0, 4, 7), 1);
        dissolveRounds_2(24, 10, 10, new Rectangle(0, 0, 3, 7), 1);
        dissolveRounds_2(30, 10, 10, new Rectangle(1, 3, 4, 8), 1);
        dissolveRounds_2(4, 10, 10, new Rectangle(14, 3, 4, 8), 1);
        dissolveRounds_2(4, 10, 10, new Rectangle(3, 14, 4, 8), 1);
        dissolveRounds_2(25, 10, 10, new Rectangle(-1, 3, 4, 8), 1);
        dissolveRounds_2(25, 10, 10, new Rectangle(-1, -1, 4, 8), 1);
        dissolveRounds_2(30, 10, 10, new Rectangle(2, -1, 4, 8), 1);
        dissolveRounds_2(2, 10, 10, new Rectangle(1, 2, -1, -3), 1);
        dissolveRounds_2(2, 10, 10, new Rectangle(1, 2, -1, 3), 1);
        dissolveRounds_2(2, 10, 10, new Rectangle(1, 2, 1, -3), 1);
        dissolveRounds_2(2, 10, 10, new Rectangle(1, 2, 0, 1), 1);
        dissolveRounds_2(2, 10, 10, new Rectangle(1, 2, 1, 0), 1);
        dissolveRounds_2(2, 10, 10, new Rectangle(1, 2, 0, 0), 1);
        // Note: Apparently, Flash Player seems to round width and height
        // in really peculiar ways when the x- and y-coordinates have
        // fractional components. Therefore, the tests involving
        // fractional parts are commented out.
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.5, 0.5, 3, 3), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.5, 0.5, 3, 4), 1);
        //dissolveRounds_2(27, 10, 10, new Rectangle(0.5, 0.5, 3, 5), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.5, 0.5, 4, 3), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.5, 0.5, 4, 4), 1);
        //dissolveRounds_2(14, 10, 10, new Rectangle(0.4, 1, 3, 4), 1);
        //dissolveRounds_2(14, 10, 10, new Rectangle(0.6, 1, 3, 4), 1);
        //dissolveRounds_2(14, 10, 10, new Rectangle(1, 1, 3.4, 4), 1);
        //dissolveRounds_2(14, 10, 10, new Rectangle(1, 1, 3.5, 4), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(1, 1, 3.6, 4), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.4, 1, 3.4, 4), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.5, 1, 3.4, 4), 1);
        //dissolveRounds_2(14, 10, 10, new Rectangle(0.6, 1, 3.4, 4), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.4, 1, 3.5, 4), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.5, 1, 3.5, 4), 1);
        //dissolveRounds_2(14, 10, 10, new Rectangle(0.6, 1, 3.5, 4), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.4, 1, 3.6, 4), 1);
        //dissolveRounds_2(18, 10, 10, new Rectangle(0.5, 1, 3.6, 4), 1);
        //dissolveRounds_2(14, 10, 10, new Rectangle(0.6, 1, 3.6, 4), 1);
        try {
            dissolveRounds_2(20, 10, 10, null, 1);
        }
        catch (e: Error) {
            trace("`null` `sourceRect` should error: " + e);
        }

        trace("");
        trace("");
        trace("------------------------------------------");
        trace("`destPoint`.");
        trace("");
        dissolveRounds_3(18, 10, 10, new Rectangle(0, 0, 3, 7), new Point(3, 5), 1);
        // This would result in a 1x6 resulting area. And Flash Player's pixelDissolve()
        // cannot handle a size where at least one of the dimensions is 1.
        //dissolveRounds_3(8, 10, 10, new Rectangle(0, 0, 3, 7), new Point(-2, -1), 1);
        dissolveRounds_3(15, 10, 10, new Rectangle(0, 0, 3, 7), new Point(-1, -1), 1);
        dissolveRounds_3(3, 10, 10, new Rectangle(0, 0, 3, 7), new Point(-10, -10), 1);
        dissolveRounds_3(3, 10, 10, new Rectangle(0, 0, 3, 7), new Point(-2, 10), 1);
        dissolveRounds_3(21, 10, 10, new Rectangle(0, 0, 3, 7), new Point(5, 4),1);

        trace("");
        trace("");
        trace("------------------------------------------");
        trace("Default parameters.");
        trace("");
        var bmd:BitmapData = new BitmapData(10, 10, false, 0xFFCCCCCC);
        // Omitting `numPixels` and `fillColor`.
        // Apparently, "numPixels:int (default = 0) — The default is 1/30 of the source area (width x height). " is wrong.
        bmd.pixelDissolve(bmd, bmd.rectangle, new Point(0, 0));
        countTraceBitmap(bmd, 0, 0xFF000000);

        trace("");
        trace("");
        trace("------------------------------------------");
        trace("`null` `sourceBitmapData`.");
        trace("");
        var bmd2:BitmapData = new BitmapData(10, 10, false, 0xFFCCCCCC);
        try {
            var result = bmd2.pixelDissolve(null, bmd2.rectangle, new Point(0, 0));
            trace("Result: " + result);
        }
        catch (e: Error) {
            trace("`sourceBitmapData` being `null` should result in error: " + e);
        }

        trace("");
        trace("");
        trace("------------------------------------------");
        trace("Wrong type for `sourceBitmapData`.");
        trace("");
        var bmd3:BitmapData = new BitmapData(10, 10, false, 0xFFCCCCCC);
        try {
            var wrongBmd = 5;
            var result = bmd3.pixelDissolve(wrongBmd, bmd3.rectangle, new Point(0, 0));
            trace("Result: " + result);
        }
        catch (e: Error) {
            trace("`sourceBitmapData` being the wrong type should result in error: " + e);
        }

        trace("");
        trace("");
        trace("------------------------------------------");
        trace("Invalid other bitmap.");
        trace("");
        var bmd4:BitmapData = new BitmapData(10, 10, false, 0xFFCCCCCC);
        var bmd5:BitmapData = new BitmapData(10, 10, false, 0xFFCFFCCC);
        bmd5.dispose();
        try {
            var result = bmd4.pixelDissolve(bmd5, bmd4.rectangle, new Point(0, 0));
            trace("Result: " + result);
        }
        catch (e: Error) {
            trace("`sourceBitmapData` being disposed() should result in error: " + e);
        }
    }

    public static function main() {
        var test = new test();
        test.test();
    }
}


import flash.display.BitmapData;
import flash.display.Bitmap;
import flash.geom.Point;
import flash.geom.Rectangle;
import flash.utils.Timer;
import flash.events.TimerEvent;

class test {

    private function add(bitmapData: BitmapData, x: Number, y: Number) {

        var destMovieClip:MovieClip = _root.createEmptyMovieClip("movieclip" + x + ", " + y, _root.getNextHighestDepth());
        destMovieClip.attachBitmap(bitmapData, _root.getNextHighestDepth());

        destMovieClip._x = x;
        destMovieClip._y = y;
        destMovieClip._width = 100;
        destMovieClip._height = 100;
    }

    function sourceHandlingOpaque(dest: BitmapData, x: Number, y: Number) {
        dest.pixelDissolve(dest, dest.rectangle, new Point(0, 0), 0, 9999, 0xFF33FF99);
        add(dest, x, y);
    }

    function sourceHandlingSemiTransparent(dest: BitmapData, x: Number, y: Number) {
        dest.pixelDissolve(dest, dest.rectangle, new Point(0, 0), 0, 9999, 0xAA5511FF);
        add(dest, x, y);
    }

    function sourceHandlingCompletelyTransparent(dest: BitmapData, x: Number, y: Number) {
        dest.pixelDissolve(dest, dest.rectangle, new Point(0, 0), 0, 9999, 0x003377FF);
        add(dest, x, y);
    }

    function sourceHandlingMixedBitmapSource(dest: BitmapData, xPos: Number, yPos: Number) {

        var sourceBitmapData4:BitmapData = new BitmapData(100, 100, true, 0xFFFFAA33);

        var x;
        var y;
        for (x = 0; x < 100; x += 1) {
            for (y = 0; y < 20; y += 1) {
                sourceBitmapData4.setPixel32(x, y, 0x00FFAA00);
            }
        }
        for (x = 0; x < 100; x += 1) {
            for (y = 20; y < 40; y += 1) {
                sourceBitmapData4.setPixel32(x, y, 0x88337777);
            }
        }

        dest.pixelDissolve(sourceBitmapData4, dest.rectangle, new Point(0, 0), 0, 9999);
        add(dest, xPos, yPos);
    }

    function destOpaque(): BitmapData {
        return new BitmapData(100, 100, true, 0xFF1199DD);
    }

    function destSemiTransparent(): BitmapData {
        return new BitmapData(100, 100, true, 0xAA44BB11);
    }

    function destCompletelyTransparent(): BitmapData {
        return new BitmapData(100, 100, true, 0x0000DD44);
    }

    public function test() {

        var x;
        var y;

        /////////////////////////////////////////////////////////////////
        // Transparency.
        /////////////////////////////////////////////////////////////////

        sourceHandlingOpaque(destOpaque(), 0, 0);
        sourceHandlingOpaque(destSemiTransparent(), 0, 100);
        sourceHandlingOpaque(destCompletelyTransparent(), 0, 200);

        // Unless there is a bug in this test or in the Ruffle implementation of
        // `BitmapData.pixelDissolve()`, `BitmapData.pixelDissolve()` might have
        // lossy conversion when the source has a semi-transparent component.
        //sourceHandlingSemiTransparent(destOpaque(), 100, 0);
        //sourceHandlingSemiTransparent(destSemiTransparent(), 100, 100);
        //sourceHandlingSemiTransparent(destCompletelyTransparent(), 100, 200);

        sourceHandlingCompletelyTransparent(destOpaque(), 200, 0);
        sourceHandlingCompletelyTransparent(destSemiTransparent(), 200, 100);
        sourceHandlingCompletelyTransparent(destCompletelyTransparent(), 200, 200);

        sourceHandlingMixedBitmapSource(destOpaque(), 300, 0);
        sourceHandlingMixedBitmapSource(destSemiTransparent(), 300, 100);
        sourceHandlingMixedBitmapSource(destCompletelyTransparent(), 300, 200);

        /////////////////////////////////////////////////////////////////
        // Source and destination offsets and areas.
        /////////////////////////////////////////////////////////////////

        var offsetBitmapData: BitmapData = new BitmapData(100, 100, false, 0xFF000000);
        var offsetBitmapData2: BitmapData = new BitmapData(100, 100, false, 0xFF000000);
        for (x = 0; x < 100; x += 1) {
            for (y = 0; y < 100; y += 1) {
                offsetBitmapData.setPixel(x, y, x + y + 0x990000);
                offsetBitmapData2.setPixel(x, y, x + y + 0x009900);
            }
        }
        offsetBitmapData2.pixelDissolve(offsetBitmapData, new Rectangle(30, 40, 35, 55), new Point(25, 10), 0, 9999);
        add(offsetBitmapData2, 400, 0);

        var offsetBitmapData3: BitmapData = new BitmapData(100, 100, false, 0xFF000000);
        var offsetBitmapData4: BitmapData = new BitmapData(100, 100, false, 0xFF000000);
        for (x = 0; x < 100; x += 1) {
            for (y = 0; y < 100; y += 1) {
                offsetBitmapData3.setPixel(x, y, x - y + 0x990000);
                offsetBitmapData4.setPixel(x, y, x + y + 0x009900);
            }
        }
        offsetBitmapData4.pixelDissolve(offsetBitmapData3, new Rectangle(30, 40, 35, 55), new Point(25, 10), 0, 9999);
        add(offsetBitmapData4, 400, 100);
    }

    public static function main() {
        var test = new test();
        test.test();
    }
}


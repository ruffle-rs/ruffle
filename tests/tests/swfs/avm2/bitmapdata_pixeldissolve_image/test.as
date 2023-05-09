
package
{
    import flash.display.Sprite;
    import flash.display.BitmapData;
    import flash.display.Bitmap;
    import flash.geom.Point;
    import flash.geom.Rectangle;
    import flash.utils.Timer;
    import flash.events.TimerEvent;

    public class test extends Sprite {

        private function add(bitmapData: BitmapData, x: Number, y: Number): void {

            var destBitmap:Bitmap = new Bitmap(bitmapData);
            destBitmap.x = x;
            destBitmap.y = y;
            this.addChild(destBitmap);
        }

        public function test() {

            var x: uint;
            var y: uint;

            /////////////////////////////////////////////////////////////////
            // Transparency.
            /////////////////////////////////////////////////////////////////

            function sourceHandlingOpaque(dest: BitmapData, x: Number, y: Number): void {
                dest.pixelDissolve(dest, dest.rect, new Point(0, 0), 0, 9999, 0xFF33FF99);
                add(dest, x, y);
            }

            function sourceHandlingSemiTransparent(dest: BitmapData, x: Number, y: Number): void {
                dest.pixelDissolve(dest, dest.rect, new Point(0, 0), 0, 9999, 0xAA5511FF);
                add(dest, x, y);
            }

            function sourceHandlingCompletelyTransparent(dest: BitmapData, x: Number, y: Number): void {
                dest.pixelDissolve(dest, dest.rect, new Point(0, 0), 0, 9999, 0x003377FF);
                add(dest, x, y);
            }

            var sourceBitmapData4:BitmapData = new BitmapData(100, 100, true, 0xFFFFAA33);
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
            function sourceHandlingMixedBitmapSource(dest: BitmapData, x: Number, y: Number): void {
                dest.pixelDissolve(sourceBitmapData4, dest.rect, new Point(0, 0), 0, 9999);
                add(dest, x, y);
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

            var destFactories: Array = [
                destOpaque,
                destSemiTransparent,
                destCompletelyTransparent
            ];
            var sourceHandlings: Array = [
                sourceHandlingOpaque,
                sourceHandlingSemiTransparent,
                sourceHandlingCompletelyTransparent,
                sourceHandlingMixedBitmapSource
            ];

            for (x = 0; x < sourceHandlings.length; x += 1) {
                for (y = 0; y < destFactories.length; y += 1) {
                    // Create a fresh destination BitmapData, call `pixelDissolve` on it with the given source, and place it at (x*100, y*100).
                    sourceHandlings[x](destFactories[y](), x*100, y*100);
                }
            }

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
    }
}


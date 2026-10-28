package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.filters.ColorMatrixFilter;
    import flash.geom.Point;
    import flash.geom.Rectangle;

    // Flash filters one more column and row of the shape than Ruffle does;
    // the stage is black so that they don't show.
    [SWF(width="110", height="60", backgroundColor="#000000", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            // The alpha offset also applies to the transparent window, so the
            // white card behind it should show through 50% black.
            var filter:ColorMatrixFilter = new ColorMatrixFilter([
                1, 0, 0, 0, 0,
                0, 1, 0, 0, 0,
                0, 0, 1, 0, 0,
                0, 0, 0, 1, 128
            ]);

            addChild(makeCard(15, 15));
            var shape:Shape = new Shape();
            shape.graphics.beginFill(0xFF0000);
            shape.graphics.drawRect(0, 0, 40, 40);
            shape.graphics.drawRect(10, 10, 20, 20);
            shape.graphics.endFill();
            shape.x = 10;
            shape.y = 10;
            shape.filters = [filter];
            addChild(shape);

            addChild(makeCard(65, 15));
            var source:BitmapData = new BitmapData(40, 40, true, 0xFFFF0000);
            source.fillRect(new Rectangle(10, 10, 20, 20), 0x00000000);
            var result:BitmapData = new BitmapData(40, 40, true, 0x00000000);
            result.applyFilter(source, source.rect, new Point(0, 0), filter);
            var bitmap:Bitmap = new Bitmap(result);
            bitmap.x = 60;
            bitmap.y = 10;
            addChild(bitmap);
        }

        private function makeCard(x:Number, y:Number):Shape {
            var card:Shape = new Shape();
            card.graphics.beginFill(0xFFFFFF);
            card.graphics.drawRect(x, y, 30, 30);
            card.graphics.endFill();
            return card;
        }
    }
}

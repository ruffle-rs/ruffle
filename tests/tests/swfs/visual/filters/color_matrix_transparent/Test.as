package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.filters.ColorMatrixFilter;
    import flash.geom.Point;
    import flash.geom.Rectangle;

    [SWF(width="148", height="40", backgroundColor="#00FFFF", frameRate="24")]
    public class Test extends Sprite {
        private static const SIZE:int = 8;
        private static const HOLE:int = 4;
        private static const COLOR:uint = 0xC08040;
        private static const GAP:int = 4;
        private static const COLUMNS:int = 6;

        public function Test() {
            var matrices:Array = [
                // Alpha offsets and factors
                [1, 0, 0, 0, 0,  0, 1, 0, 0, 0,  0, 0, 1, 0, 0,  0, 0, 0, 1, 128],
                [1, 0, 0, 0, 0,  0, 1, 0, 0, 0,  0, 0, 1, 0, 0,  0, 0, 0, 1, 255],
                [1, 0, 0, 0, 0,  0, 1, 0, 0, 0,  0, 0, 1, 0, 0,  0, 0, 0, 1, -128],
                [1, 0, 0, 0, 0,  0, 1, 0, 0, 0,  0, 0, 1, 0, 0,  0, 0, 0, 0.25, 64],
                // Color offsets
                [1, 0, 0, 0, 64,  0, 1, 0, 0, 32,  0, 0, 1, 0, 16,  0, 0, 0, 1, 0],
                [1, 0, 0, 0, 64,  0, 1, 0, 0, 32,  0, 0, 1, 0, 16,  0, 0, 0, 1, 128],
                [1, 0, 0, 0, 64,  0, 1, 0, 0, 32,  0, 0, 1, 0, 16,  0, 0, 0, 1, 255],
                [1, 0, 0, 0, 200,  0, 1, 0, 0, 0,  0, 0, 1, 0, 0,  0, 0, 0, 1, 255],
                [1, 0, 0, 0, 0,  0, 1, 0, 0, 200,  0, 0, 1, 0, 0,  0, 0, 0, 1, 255],
                [1, 0, 0, 0, 0,  0, 1, 0, 0, 0,  0, 0, 1, 0, 200,  0, 0, 0, 1, 255],
                [1, 0, 0, 0, 200,  0, 1, 0, 0, 100,  0, 0, 1, 0, 50,  0, 0, 0, 1, 255],
                [1, 0, 0, 0, -64,  0, 1, 0, 0, -64,  0, 0, 1, 0, -64,  0, 0, 0, 1, 255],
                [0, 0, 0, 0, 255,  0, 0, 0, 0, 255,  0, 0, 0, 0, 255,  0, 0, 0, 0, 255],
                // Color factors
                [0, 0, 0, 1, 0,  0, 1, 0, 0, 0,  0, 0, 1, 0, 0,  0, 0, 0, 1, 255],
                [1, 0, 0, 0, 0,  0, 1, 0, 0, 0,  0, 0, 1, 0, 0,  1, 0, 0, 0, 0],
                [0, 0, 1, 0, 0,  0, 1, 0, 0, 64,  1, 0, 0, 0, 128,  0, 0, 0, 1, 128],
                [0.3, 0.59, 0.11, 0, 0,  0.3, 0.59, 0.11, 0, 0,  0.3, 0.59, 0.11, 0, 0,  0, 0, 0, 1, 128],
                [-1, 0, 0, 0, 255,  0, -1, 0, 0, 255,  0, 0, -1, 0, 255,  0, 0, 0, 1, 255]
            ];

            for (var i:int = 0; i < matrices.length; i++) {
                var filter:ColorMatrixFilter = new ColorMatrixFilter(matrices[i]);
                var px:int = GAP + (i % COLUMNS) * 2 * (SIZE + GAP);
                var py:int = GAP + int(i / COLUMNS) * (SIZE + GAP);
                addShape(px, py, filter);
                addBitmap(px + SIZE + GAP, py, filter);
            }
        }

        private function addShape(px:int, py:int, filter:ColorMatrixFilter):void {
            var shape:Shape = new Shape();
            shape.graphics.beginFill(COLOR);
            shape.graphics.drawRect(0, 0, SIZE, SIZE);
            shape.graphics.drawRect((SIZE - HOLE) / 2, (SIZE - HOLE) / 2, HOLE, HOLE);
            shape.graphics.endFill();
            shape.x = px;
            shape.y = py;
            shape.filters = [filter];
            addChild(shape);
        }

        private function addBitmap(px:int, py:int, filter:ColorMatrixFilter):void {
            var source:BitmapData = new BitmapData(SIZE, SIZE, true, 0xFF000000 | COLOR);
            source.fillRect(new Rectangle((SIZE - HOLE) / 2, (SIZE - HOLE) / 2, HOLE, HOLE), 0);
            var result:BitmapData = new BitmapData(SIZE, SIZE, true, 0);
            result.applyFilter(source, source.rect, new Point(0, 0), filter);
            var bitmap:Bitmap = new Bitmap(result);
            bitmap.x = px;
            bitmap.y = py;
            addChild(bitmap);
        }
    }
}

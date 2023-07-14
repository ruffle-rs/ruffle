package {
    import flash.display.BitmapData;
    import flash.geom.Rectangle;

    public class Test {
        private var bData: BitmapData;

        private static const WHITE: uint = 0xffffffff;
        private static const RED: uint = 0xffff0000;
        private static const BLUE: uint = 0xff00ff00;
        private static const TRANSPARENT_WHITE: uint = 0x80ffffff;

        public function Test() {
            bData = new BitmapData(3, 3, true, 0);

            test(bData.rect, Vector.<uint>([WHITE, WHITE, WHITE, RED, RED, RED, TRANSPARENT_WHITE, TRANSPARENT_WHITE, TRANSPARENT_WHITE]));

            // Extra entries okay
            test(bData.rect, Vector.<uint>([RED, RED, RED, WHITE, WHITE, WHITE, TRANSPARENT_WHITE, TRANSPARENT_WHITE, TRANSPARENT_WHITE, RED]));

            // Out-of-bounds-rect okay
            test(new Rectangle(-10, -10, 100, 100), Vector.<uint>([WHITE, WHITE, WHITE, RED, RED, RED, BLUE, BLUE, BLUE]));

            // No size
            test(new Rectangle(), Vector.<uint>([]));

            // No wrapping behavior on rectangle
            test(new Rectangle(4294967296, 0, 3, 3), Vector.<uint>([BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE]));

            // == ERRORS ==
            // Null rect
            test(null, null);

            // Null vec
            test(bData.rect, null);

            // Not enough data
            test(bData.rect, Vector.<uint>([WHITE, WHITE, WHITE, RED, RED, RED, RED, RED]));
        }

        function test(rectangle: Rectangle, data: Vector.<uint>):void {
            try {
                trace("// bData.setVector(" + rectangle + ", " + (data != null ? "[" + data + "]" : "null") + ");");
                bData.setVector(rectangle, data);
            } catch(e:*) {
                trace("Error: " + e);
            }

            trace("Bitmap: [" + bData.getVector(bData.rect) + "]");
            trace("");
        }
    }
}

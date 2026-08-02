// compiled with mxmlc

package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Rectangle;

    // Scaled below its own corners: the centre vanishes and the corners split what is left
    // in proportion to their authored sizes, so an asymmetric grid stays asymmetric.
    public class Test extends MovieClip {
        private static const SIZE:Number = 100;

        public function Test() {
            // Symmetric corners: a proportional and an even split agree here.
            row(0, new Rectangle(30, 30, 40, 40), 0.4);

            // Asymmetric on x: a 10px corner against a 40px one, which splits 1:4.
            row(1, new Rectangle(10, 30, 50, 40), 0.4);

            // Asymmetric on both axes.
            row(2, new Rectangle(10, 10, 50, 50), 0.4);

            // Collapsed hard, well under the corners.
            row(3, new Rectangle(30, 30, 40, 40), 0.15);

            // Above the threshold, as the reference.
            row(4, new Rectangle(10, 30, 50, 40), 1.6);
        }

        private function row(index:int, grid:Rectangle, scale:Number):void {
            var y:Number = 10 + index * 70;

            var art:Shape = paint();
            art.scale9Grid = grid;
            art.scaleX = scale;
            art.scaleY = scale;
            art.x = 20;
            art.y = y;
            addChild(art);

            var plain:Shape = paint();
            plain.scaleX = scale;
            plain.scaleY = scale;
            plain.x = 200;
            plain.y = y;
            addChild(plain);
        }

        private function paint():Shape {
            var s:Shape = new Shape();
            var cols:Array = [0xE04040, 0xE0E040, 0x40E040, 0x40E0E0, 0x4040E0];
            var edges:Array = [0, 10, 30, 70, 90, 100];
            for (var iy:int = 0; iy < 5; iy++) {
                for (var ix:int = 0; ix < 5; ix++) {
                    s.graphics.beginFill(uint(cols[(ix + iy) % 5]));
                    s.graphics.drawRect(edges[ix], edges[iy],
                        edges[ix + 1] - edges[ix], edges[iy + 1] - edges[iy]);
                    s.graphics.endFill();
                }
            }
            return s;
        }
    }
}

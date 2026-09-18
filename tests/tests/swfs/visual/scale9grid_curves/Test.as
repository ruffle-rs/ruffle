package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Rectangle;

    // Curves are not subdivided at the grid lines: a control point goes through the same
    // map as an anchor, so a curve crossing a line stays one curve with a moved control
    // point.
    [SWF(width="360", height="320", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const W:Number = 120;
        private static const H:Number = 80;

        public function Test() {
            // Corner arcs inside the pinned corners: radius must survive the stretch.
            pair(0, new Rectangle(28, 24, W - 56, H - 48), 2.6, 1.0);

            // Grid lines drawn through the arcs, so a curve straddles a line.
            pair(1, new Rectangle(12, 12, W - 24, H - 24), 2.6, 1.0);

            // Squeezed instead of stretched.
            pair(2, new Rectangle(28, 24, W - 56, H - 48), 0.55, 1.0);
        }

        private function pair(index:int, grid:Rectangle, sx:Number, sy:Number):void {
            var y:Number = 15 + index * 100;

            var art:Shape = paint();
            art.scale9Grid = grid;
            art.scaleX = sx;
            art.scaleY = sy;
            art.x = 15;
            art.y = y;
            addChild(art);

            var plain:Shape = paint();
            plain.scaleX = sx;
            plain.scaleY = sy;
            plain.x = 15;
            plain.y = y + 42;
            addChild(plain);
        }

        // A rounded rectangle, plus a long curve whose control point sits in the centre.
        private function paint():Shape {
            var s:Shape = new Shape();
            var r:Number = 24;

            s.graphics.beginFill(0x3060C0);
            s.graphics.moveTo(r, 0);
            s.graphics.lineTo(W - r, 0);
            s.graphics.curveTo(W, 0, W, r);
            s.graphics.lineTo(W, H - r);
            s.graphics.curveTo(W, H, W - r, H);
            s.graphics.lineTo(r, H);
            s.graphics.curveTo(0, H, 0, H - r);
            s.graphics.lineTo(0, r);
            s.graphics.curveTo(0, 0, r, 0);
            s.graphics.endFill();

            s.graphics.beginFill(0xE0C040);
            s.graphics.moveTo(8, H / 2);
            s.graphics.curveTo(W / 2, H / 2 - 26, W - 8, H / 2);
            s.graphics.curveTo(W / 2, H / 2 - 10, 8, H / 2);
            s.graphics.endFill();

            return s;
        }
    }
}

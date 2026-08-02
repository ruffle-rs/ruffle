// compiled with mxmlc

package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.filters.GlowFilter;
    import flash.geom.Rectangle;

    // Slicing happens before the object is captured into a bitmap, so a cached copy must
    // match the uncached one pixel for pixel. A filter forces the cache on without asking,
    // and a fractional scale decides the resolution the capture is taken at.
    public class Test extends MovieClip {
        private static const SIZE:Number = 60;
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);

        public function Test() {
            plain(0, 3);
            cached(1, 3);
            plain(2, 2.5);
            cached(3, 2.5);
            filtered(4, 3);
            scrolled(5, 3);
        }

        private function plain(index:int, scale:Number):void {
            place(notch(), index, scale);
        }

        private function cached(index:int, scale:Number):void {
            var s:Shape = notch();
            s.cacheAsBitmap = true;
            place(s, index, scale);
        }

        // A filter turns the bitmap cache on whether or not it was asked for.
        private function filtered(index:int, scale:Number):void {
            var s:Shape = notch();
            s.filters = [new GlowFilter(0x40C0FF, 1, 6, 6, 2, 1)];
            place(s, index, scale);
        }

        // A scroll rect clips against the sliced result, not the authored one.
        private function scrolled(index:int, scale:Number):void {
            var s:Shape = notch();
            var holder:Sprite = new Sprite();
            holder.addChild(s);
            s.scale9Grid = GRID;
            s.scaleX = scale;
            holder.scrollRect = new Rectangle(10, 10, 150, 45);
            holder.x = 20;
            holder.y = 20 + index * 70;
            addChild(holder);
        }

        private function place(s:Shape, index:int, scale:Number):void {
            s.scale9Grid = GRID;
            s.scaleX = scale;
            s.x = 20;
            s.y = 20 + index * 70;
            addChild(s);
        }

        private function notch():Shape {
            var s:Shape = new Shape();
            s.graphics.beginFill(0xC02020);
            s.graphics.moveTo(0, 0);
            s.graphics.lineTo(SIZE, 0);
            s.graphics.lineTo(SIZE, SIZE);
            s.graphics.lineTo(SIZE - 20, SIZE);
            s.graphics.lineTo(SIZE - 20, SIZE - 20);
            s.graphics.lineTo(20, SIZE - 20);
            s.graphics.lineTo(20, SIZE);
            s.graphics.lineTo(0, SIZE);
            s.graphics.endFill();
            return s;
        }
    }
}

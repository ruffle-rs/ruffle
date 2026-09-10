package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.geom.Matrix;
    import flash.geom.Rectangle;

    // BitmapData.draw leaves the source's own transform off, and the grid goes with it: the
    // source is drawn whole however the draw matrix stretches it. A gridded child of the
    // source still slices, since its own transform is applied. FLVPlayback's skins depend on
    // this, drawing a gridded border nine times to slice it themselves.
    [SWF(width="200", height="500", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 60;
        private static const CORNER:Number = 20;
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);
        private static const SCALE:Number = 3;

        public function Test() {
            // Sliced the ordinary way, for the corners the other rows are read against.
            var live:Shape = art();
            live.scale9Grid = GRID;
            live.scaleX = SCALE;
            place(live, 0);

            // Source of the draw, so neither its scaleX nor its grid reaches the bitmap.
            place(shot(art2(), null, SIZE), 1);

            // Same, with the stretch asked for by the draw rather than by the source.
            place(shot(art2(), new Matrix(SCALE, 0, 0, 1), SIZE * SCALE), 2);

            // A child of the source: its transform is applied, so its grid applies too.
            var holder:Sprite = new Sprite();
            var kid:Shape = art();
            kid.scale9Grid = GRID;
            kid.scaleX = SCALE;
            holder.addChild(kid);
            place(shot(holder, null, SIZE * SCALE), 3);

            // The grid one level up: drawing the gridded container leaves its plain child
            // unsliced too, and so does drawing that child directly.
            place(shot(gridded(), null, SIZE), 4);
            place(shot(gridded().getChildAt(0), null, SIZE), 5);

            // The same pair rendered live for comparison: the child slices by inheritance.
            var live2:Sprite = gridded();
            place(live2, 6);
        }

        /** A gridded, scaled container whose only art is a plain Shape child. */
        private function gridded():Sprite {
            var holder:Sprite = new Sprite();
            holder.addChild(art());
            holder.scale9Grid = GRID;
            holder.scaleX = SCALE;
            return holder;
        }

        /** A gridded shape already carrying the scale, so a draw has something to ignore. */
        private function art2():Shape {
            var s:Shape = art();
            s.scale9Grid = GRID;
            s.scaleX = SCALE;
            return s;
        }

        private function shot(source:*, matrix:Matrix, width:Number):Bitmap {
            var bmd:BitmapData = new BitmapData(width, SIZE, true, 0x00000000);
            bmd.draw(source, matrix);
            return new Bitmap(bmd);
        }

        private function place(art:*, index:int):void {
            art.x = 10;
            art.y = 10 + index * 70;
            addChild(art);
        }

        /** Corner squares in a second colour, so a kept corner is 20 wide and a scaled one 60. */
        private function art():Shape {
            var s:Shape = new Shape();
            s.graphics.beginFill(0x3366AA);
            s.graphics.drawRect(0, 0, SIZE, SIZE);
            s.graphics.endFill();
            s.graphics.beginFill(0xEE9933);
            s.graphics.drawRect(0, 0, CORNER, CORNER);
            s.graphics.drawRect(SIZE - CORNER, 0, CORNER, CORNER);
            s.graphics.drawRect(0, SIZE - CORNER, CORNER, CORNER);
            s.graphics.drawRect(SIZE - CORNER, SIZE - CORNER, CORNER, CORNER);
            s.graphics.endFill();
            return s;
        }
    }
}

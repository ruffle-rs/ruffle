package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.geom.Rectangle;

    // A gridded object serving as a scripted mask scales plainly, like a timeline masker,
    // for both the stencil kind and the alpha kind (cacheAsBitmap on masker and maskee).
    // The masker's outer bands reveal a white backdrop, so the silhouette reads 60/60/60
    // plain against 20/140/20 had the mask sliced; the third row shows the same art as an
    // ordinary child, which must slice. The last two rows put the grid one level up: a
    // gridded container as the stencil masker, and a container's plain child as the alpha
    // masker -- the child neither inherits while masking.
    [SWF(width="210", height="360", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);

        public function Test() {
            row(10, false);
            row(80, true);

            var control:Shape = masker();
            control.x = 10;
            control.y = 150;
            addChild(control);

            containerRow(220, false);
            containerRow(290, true);
        }

        // The grid sits on a container; the masker is the container (stencil) or its
        // plain child (alpha). Either way nothing may slice while the mask draws.
        private function containerRow(y:Number, alpha:Boolean):void {
            var backdrop:Shape = new Shape();
            backdrop.graphics.beginFill(0xFFFFFF);
            backdrop.graphics.drawRect(0, 0, 190, 60);
            backdrop.graphics.endFill();
            backdrop.x = 10;
            backdrop.y = y;

            var holder:Sprite = new Sprite();
            var kid:Shape = masker();
            kid.scale9Grid = null;
            kid.scaleX = 1;
            holder.addChild(kid);
            holder.scale9Grid = GRID;
            holder.scaleX = 3;
            holder.x = 10;
            holder.y = y;

            if (alpha) {
                backdrop.cacheAsBitmap = true;
                kid.cacheAsBitmap = true;
                backdrop.mask = kid;
            } else {
                backdrop.mask = holder;
            }
            addChild(backdrop);
            addChild(holder);
        }

        private function row(y:Number, alpha:Boolean):void {
            var backdrop:Shape = new Shape();
            backdrop.graphics.beginFill(0xFFFFFF);
            backdrop.graphics.drawRect(0, 0, 190, 60);
            backdrop.graphics.endFill();
            backdrop.x = 10;
            backdrop.y = y;

            var m:Shape = masker();
            m.x = 10;
            m.y = y;
            if (alpha) {
                backdrop.cacheAsBitmap = true;
                m.cacheAsBitmap = true;
            }
            backdrop.mask = m;
            addChild(backdrop);
            addChild(m);
        }

        private function masker():Shape {
            var s:Shape = new Shape();
            s.graphics.beginFill(0xFF0000);
            s.graphics.drawRect(0, 0, 20, 60);
            s.graphics.endFill();
            s.graphics.beginFill(0x00FF00);
            s.graphics.drawRect(20, 0, 20, 4);
            s.graphics.endFill();
            s.graphics.beginFill(0x0000FF);
            s.graphics.drawRect(40, 0, 20, 60);
            s.graphics.endFill();
            s.scale9Grid = GRID;
            s.scaleX = 3;
            return s;
        }
    }
}

package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.geom.Rectangle;

    // A Shape child that carries a grid of its own, inside a gridded container. A child
    // whose own grid can slice uses it and the parent's never touches it; one whose grid
    // cannot slice -- rotation gates a grid on the object's own matrix -- still inherits,
    // but Flash Player throws the child's matrix away and remaps the raw coordinates.
    // Rows: own grid at identity (renders plain: the child's grid wins and a scale-1 remap
    // is the identity), own grid rotated 10 degrees (axis-aligned bands through the
    // parent's map), no grid rotated 10 degrees (the ordinary folded remap).
    [SWF(width="260", height="270", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);
        private static const CHILD_GRID:Rectangle = new Rectangle(10, 10, 40, 40);

        public function Test() {
            place(build(true, 0), 10);
            place(build(true, 10), 95);
            place(build(false, 10), 180);
        }

        private function place(s:Sprite, y:Number):void {
            s.x = 30;
            s.y = y;
            addChild(s);
        }

        private function build(ownGrid:Boolean, rot:Number):Sprite {
            var container:Sprite = new Sprite();
            container.graphics.beginFill(0x303030);
            container.graphics.drawRect(0, 0, 60, 60);
            container.graphics.endFill();

            var child:Shape = new Shape();
            band(child, 0, 0xFF0000);
            band(child, 20, 0x00FF00);
            band(child, 40, 0x0000FF);
            if (ownGrid) {
                child.scale9Grid = CHILD_GRID;
            }
            child.rotation = rot;
            container.addChild(child);

            container.scale9Grid = GRID;
            container.scaleX = 3;
            return container;
        }

        private function band(s:Shape, x:Number, color:uint):void {
            s.graphics.beginFill(color);
            s.graphics.drawRect(x, 0, 20, 60);
            s.graphics.endFill();
        }
    }
}

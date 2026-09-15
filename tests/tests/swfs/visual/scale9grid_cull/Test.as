package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.geom.Rectangle;

    // Sliced geometry is not contained in the plainly scaled bounds, so it must not be
    // culled by them. The child's own grid is gated off by rotation and its matrix --
    // translation included -- is discarded, so its plain bounds sit far off stage while
    // the render lands at the origin; the translated extents still widen the container's
    // bounds, which is what stretches the middle band past the stage edge.
    [SWF(width="260", height="80", backgroundColor="#101010")]
    public class Test extends MovieClip {
        public function Test() {
            var container:Sprite = new Sprite();
            container.graphics.beginFill(0x303030);
            container.graphics.drawRect(0, 0, 60, 60);
            container.graphics.endFill();

            var child:Shape = new Shape();
            band(child, 0, 0xFF0000);
            band(child, 20, 0x00FF00);
            band(child, 40, 0x0000FF);
            child.scale9Grid = new Rectangle(10, 10, 40, 40);
            child.rotation = 10;
            child.x = 500;
            container.addChild(child);

            container.scale9Grid = new Rectangle(20, 20, 20, 20);
            container.scaleX = 3;
            container.x = 10;
            container.y = 10;
            addChild(container);
        }

        private function band(s:Shape, x:Number, color:uint):void {
            s.graphics.beginFill(color);
            s.graphics.drawRect(x, 0, 20, 60);
            s.graphics.endFill();
        }
    }
}

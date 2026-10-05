package {
    import flash.display.DisplayObject;
    import flash.display.DisplayObjectContainer;
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.events.Event;
    import flash.geom.Rectangle;
    import flash.net.URLRequest;

    // A quadratic baked into a DefineShape goes through the same map as one drawn through
    // Graphics: control points are remapped like anchors, the curve is never split at a
    // grid line. The control point sits inside the left corner region, where the two
    // models disagree by ~7px at 3x. Rows: baked art gridded by the tag, baked art
    // gridded through the setter, baked art ungridded, baked art at 1x, and the same art
    // drawn at runtime with the setter. Rows one, two and five must match.
    [SWF(width="200", height="370", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private var loader:Loader = new Loader();

        public function Test() {
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onLoad);
            loader.load(new URLRequest("art.swf"));
        }

        private function onLoad(e:Event):void {
            addChild(loader);
            var art:DisplayObjectContainer = DisplayObjectContainer(loader.content);
            var baked:DisplayObject = art.getChildAt(1);
            baked.scale9Grid = new Rectangle(20, 20, 20, 20);
            baked.scaleX = 3;

            var drawn:Shape = new Shape();
            paint(drawn);
            drawn.scale9Grid = new Rectangle(20, 20, 20, 20);
            drawn.scaleX = 3;
            drawn.x = 10;
            drawn.y = 290;
            addChild(drawn);
        }

        private function paint(s:Shape):void {
            band(s, 0, 0xFF0000);
            band(s, 20, 0x00FF00);
            band(s, 40, 0x0000FF);
            s.graphics.beginFill(0xFFFFFF);
            s.graphics.moveTo(0, 60);
            s.graphics.curveTo(10, 0, 60, 60);
            s.graphics.lineTo(0, 60);
            s.graphics.endFill();
        }

        private function band(s:Shape, x:Number, color:uint):void {
            s.graphics.beginFill(color);
            s.graphics.drawRect(x, 0, 20, 4);
            s.graphics.endFill();
        }
    }
}

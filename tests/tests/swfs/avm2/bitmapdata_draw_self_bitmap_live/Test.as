package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.Shape;
    import flash.display.Sprite;

    // Tests drawing a container into a BitmapData, when that container holds a Bitmap
    // of the same BitmapData: the Bitmap shows the target as already modified by the
    // objects drawn before it in the same draw() call.
    [SWF(width="100", height="100")]
    public class Test extends Sprite {
        private function hex(v:uint):String {
            var s:String = v.toString(16).toUpperCase();
            while (s.length < 8) s = "0" + s;
            return s;
        }

        private function run(name:String, belowColor:uint, aboveColor:uint):void {
            var bmd:BitmapData = new BitmapData(40, 10, true, 0x80FF0000);
            var container:Sprite = new Sprite();

            var below:Shape = new Shape();
            below.graphics.beginFill(belowColor & 0xFFFFFF, (belowColor >>> 24) / 255);
            below.graphics.drawRect(0, 0, 20, 10);
            container.addChild(below);

            container.addChild(new Bitmap(bmd));

            var above:Shape = new Shape();
            above.graphics.beginFill(aboveColor & 0xFFFFFF, (aboveColor >>> 24) / 255);
            above.graphics.drawRect(30, 0, 10, 10);
            container.addChild(above);

            bmd.draw(container);
            trace(name + ": under shape below=" + hex(bmd.getPixel32(5, 5)) +
                " untouched=" + hex(bmd.getPixel32(25, 5)) +
                " under shape above=" + hex(bmd.getPixel32(35, 5)));
        }

        public function Test() {
            run("opaque", 0xFFFFFFFF, 0xFF0000FF);
            run("transparent", 0x00FFFFFF, 0x000000FF);
            run("black", 0xFF000000, 0xFF00FF00);
        }
    }
}

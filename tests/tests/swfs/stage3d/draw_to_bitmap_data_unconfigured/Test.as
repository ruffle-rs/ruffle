package {
    import flash.display.BitmapData;
    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DRenderMode;
    import flash.events.Event;

    // drawToBitmapData before configureBackBuffer has been called.
    [SWF(width="100", height="50", backgroundColor="#000000")]
    public class Test extends Sprite {
        private var context:Context3D;

        public function Test() {
            var s3d:Stage3D = stage.stage3Ds[0];
            s3d.addEventListener(Event.CONTEXT3D_CREATE, onContext);
            s3d.requestContext3D(Context3DRenderMode.AUTO);
        }

        private function onContext(e:Event):void {
            context = Stage3D(e.target).context3D;

            read("no clear");

            try {
                context.clear(0.2, 0.4, 0.6, 1);
                trace("clear: no error");
            } catch (e:Error) {
                trace("clear: " + Object(e).constructor);
                trace(e.getStackTrace());
            }
            read("after clear");

            context.configureBackBuffer(50, 50, 0, false);
            read("after configure");
        }

        private function read(name:String):void {
            var bmp:BitmapData = new BitmapData(50, 50, true, 0xff808080);
            try {
                context.drawToBitmapData(bmp);
            } catch (e:Error) {
                trace(name + ": " + Object(e).constructor);
                trace(e.getStackTrace());
                return;
            }
            trace(name + ": " + hex(bmp.getPixel32(10, 25)));
        }

        private function hex(color:uint):String {
            var s:String = color.toString(16);
            while (s.length < 8) {
                s = "0" + s;
            }
            return s;
        }
    }
}

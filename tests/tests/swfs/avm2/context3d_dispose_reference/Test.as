package {
    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DRenderMode;
    import flash.events.Event;

    public class Test extends Sprite {
        private var s3d:Stage3D;
        private var creates:int = 0;
        private var oldContext:Context3D;

        public function Test() {
            s3d = stage.stage3Ds[0];
            s3d.addEventListener(Event.CONTEXT3D_CREATE, onContext);
            s3d.requestContext3D(Context3DRenderMode.AUTO);
        }

        private function onContext(e:Event):void {
            creates++;
            if (creates == 1) {
                oldContext = s3d.context3D;
                oldContext.dispose(true);
            } else if (creates == 2) {
                try {
                    oldContext.createVertexBuffer(3, 3);
                    trace("old context still usable (no error)");
                } catch (err:Error) {
                    trace("errorID: " + err.errorID);
                }
            }
        }
    }
}

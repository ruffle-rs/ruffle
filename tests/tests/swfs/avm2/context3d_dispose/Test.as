package {
    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DRenderMode;
    import flash.events.Event;

    // dispose(true) makes Stage3D dispatch a fresh context3DCreate; dispose(false)
    // leaves it without a context. Dispose true on the first create (a second must
    // follow), false on the second (no third).
    public class Test extends Sprite {
        private var s3d:Stage3D;
        private var creates:int = 0;

        public function Test() {
            s3d = stage.stage3Ds[0];
            s3d.addEventListener(Event.CONTEXT3D_CREATE, onContext);
            s3d.requestContext3D(Context3DRenderMode.AUTO);
        }

        private function onContext(e:Event):void {
            creates++;
            trace("context3DCreate #" + creates);

            var context:Context3D = s3d.context3D;
            if (creates == 1) {
                context.dispose(true);
            } else if (creates == 2) {
                context.dispose(false);
                trace("context3D after dispose(false): " +
                    (s3d.context3D == null ? "null" : "set"));
            }
        }
    }
}

package {
    import flash.display.BitmapData;
    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DTextureFormat;
    import flash.display3D.Context3DRenderMode;
    import flash.events.Event;

    public class Test extends Sprite {
        public function Test() {
            var stage3D:Stage3D = stage.stage3Ds[0];
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            stage3D.requestContext3D(Context3DRenderMode.AUTO);
        }

        private function contextCreated(event:Event):void {
            var context:Context3D = Stage3D(event.target).context3D;
            var texture = context.createTexture(
                4,
                4,
                Context3DTextureFormat.COMPRESSED_ALPHA,
                false
            );
            texture.uploadFromBitmapData(new BitmapData(4, 4, false, 0xff0000));
            trace("uploadFromBitmapData completed");
        }
    }
}

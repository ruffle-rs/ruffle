package {
    import flash.display.MovieClip;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DRenderMode;
    import flash.events.Event;
    import flash.utils.ByteArray;

    public class Test extends MovieClip {

        private var stage3D:Stage3D;

        private var renderContext:Context3D;

        public function Test() {
            super();
            this.stage3D = this.stage.stage3Ds[0];
            this.stage3D.addEventListener(Event.CONTEXT3D_CREATE,this.contextCreated);
            this.stage3D.requestContext3D(Context3DRenderMode.AUTO,"standard");
        }

        private function contextCreated(event:Event):void {
            this.renderContext = Stage3D(event.target).context3D;
            var circleATF:ByteArray = new Test_CIRCLE_ATF();

            try {
                var atfTexture1 = this.renderContext.createTexture(256,256,"bgra",false);
                atfTexture1.uploadCompressedTextureFromByteArray(circleATF,0);
                trace("#1 should not succeed");
            } catch(e:Error) {
                trace("#1: " + e.errorID);
            }

            try {
                var atfTexture2 = this.renderContext.createCubeTexture(512,"bgra",false);
                atfTexture2.uploadCompressedTextureFromByteArray(circleATF,0);
                trace("#2 should not succeed");
            } catch(e:Error) {
                trace("#2: " + e.errorID);
            }

            try {
                var atfTexture3 = this.renderContext.createCubeTexture(256,"bgra",false);
                atfTexture3.uploadCompressedTextureFromByteArray(circleATF,0);
                trace("#3 should not succeed");
            } catch(e:Error) {
                trace("#3: " + e.errorID);
            }
        }
    }
}


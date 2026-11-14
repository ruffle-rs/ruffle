package flash.display3D.textures {
    import flash.media.Camera;
    import flash.net.NetStream;

    [API("706")]
    [Ruffle(Abstract)]
    public final class VideoTexture extends TextureBase {
        public function get videoWidth():int {
            return 0;
        }

        public function get videoHeight():int {
            return 0;
        }

        public function attachCamera(theCamera:Camera):void {
        }

        public function attachNetStream(netStream:NetStream):void {
        }
    }
}

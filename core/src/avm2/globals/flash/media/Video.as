package flash.media
{
    import flash.display.DisplayObject
    import flash.net.NetStream
    
    public class Video extends DisplayObject
    {
        private var _videoWidth: int;
        private var _videoHeight: int;
        
        public function Video(width: int = 320, height: int = 240) {
            this._videoWidth = width;
            this._videoHeight = height;
        }
        
        public function get videoWidth():int {
            return this._videoWidth;
        }
        
        public function get videoHeight():int {
            return this._videoHeight;
        }

        public native function attachNetStream(netStream: NetStream);
    }
}

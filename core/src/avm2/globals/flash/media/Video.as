package flash.media
{
    import __ruffle__.stub_method;

    import flash.display.DisplayObject
    import flash.net.NetStream
    
    [Ruffle(InstanceAllocator)]
    public class Video extends DisplayObject
    {
        private var _deblocking: int;
        private var _smoothing: Boolean;
        private var _videoWidth: int;
        private var _videoHeight: int;

        public function Video(width: int = 320, height: int = 240) {
            if (width < 0 || height < 0) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            this._videoWidth = width;
            this._videoHeight = height;
            this.init(width, height);
        }

        private native function init(width: int, height: int);

        public function get deblocking():int {
            return this._deblocking;
        }

        public function set deblocking(value:int):void {
            this._deblocking = value;
        }

        public function get smoothing():Boolean {
            return this._smoothing;
        }

        public function set smoothing(value:Boolean):void {
            this._smoothing = value;
        }

        public function get videoWidth():int {
            return this._videoWidth;
        }

        public function get videoHeight():int {
            return this._videoHeight;
        }

        public native function attachNetStream(netStream: NetStream);

        public function clear():void {
            stub_method("flash.media.Video", "clear");
        }
    }
}

package flash.media {
    import flash.display.DisplayObject;
    import flash.net.NetStream;

    [Ruffle(InstanceAllocator)]
    public class Video extends DisplayObject {
        private var _deblocking:int;
        private var _smoothing:Boolean;

        public function Video(width:int = 320, height:int = 240) {
            this.init(width, height);
        }

        private native function init(width:int, height:int):void;

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

        public native function get videoWidth():int;

        public native function get videoHeight():int;

        public function attachCamera(camera:Camera):void {
        }

        public native function attachNetStream(netStream:NetStream):void;

        public function clear():void {
        }
    }
}

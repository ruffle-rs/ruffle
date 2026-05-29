package flash.media {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;

    import flash.display.DisplayObject
    import flash.net.NetStream

    [Ruffle(InstanceAllocator)]
    public class Video extends DisplayObject {
        private var _deblocking:int;
        private var _smoothing:Boolean;
        private var _videoWidth:int;
        private var _videoHeight:int;

        public function Video(width:int = 320, height:int = 240) {
            this.init(width, height);
        }

        private native function init(width:int, height:int);

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
            stub_getter("flash.media.Video", "videoWidth");
            return 0;
        }

        public function get videoHeight():int {
            stub_getter("flash.media.Video", "videoHeight");
            return 0;
        }

        public native function attachNetStream(netStream:NetStream):void;

        public function clear():void {
            stub_method("flash.media.Video", "clear");
        }
    }
}

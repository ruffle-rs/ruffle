package flash.media {
    import flash.events.EventDispatcher;
    import flash.geom.Point;
    import flash.geom.Rectangle;
    import flash.net.NetStream;

    // Docs are inaccurate here, StageVideo is available in SWF 10, but not
    // available in FP 10,0,42,34.
    [API("667")]
    [Ruffle(Abstract)]
    public class StageVideo extends EventDispatcher {
        private var _depth:int = 0;
        private var _pan:Point = new Point(0, 0);
        private var _zoom:Point = new Point(1, 1);
        private var _viewPort:Rectangle = new Rectangle(0, 0, 0, 0);

        [API("680")]
        public function attachCamera(theCamera:Camera):void {
        }

        public function attachNetStream(netStream:NetStream):void {
        }

        public function get colorSpaces():Vector.<String> {
            return new <String>["BT.601"];
        }

        public function get depth():int {
            return this._depth;
        }
        public function set depth(value:int):void {
            this._depth = value;
        }

        public function get pan():Point {
            return this._pan;
        }
        public function set pan(value:Point):void {
            this._pan = value;
        }

        public function get videoHeight():int {
            return 0;
        }

        public function get videoWidth():int {
            return 0;
        }

        public function get viewPort():Rectangle {
            return this._viewPort;
        }
        public function set viewPort(value:Rectangle):void {
            this._viewPort = value;
        }

        public function get zoom():Point {
            return this._zoom;
        }
        public function set zoom(value:Point):void {
            this._zoom = value;
        }
    }
}

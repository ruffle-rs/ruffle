package flash.media {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import flash.geom.Rectangle;
    import flash.display.BitmapData;

    public final class Camera extends EventDispatcher {
        private var _bandwidth:int = 16384;
        private var _fps:Number = 15;
        private var _height:int = 120;
        private var _index:int = 0;
        private var _keyFrameInterval:int = 15;
        private var _loopback:Boolean = false;
        private var _motionLevel:int = 50;
        private var _motionTimeout:int = 2000;
        private var _name:String = "";
        private var _quality:int = 0;
        private var _width:int = 160;

        [API("682")]
        public function copyToByteArray(rect:Rectangle, destination:ByteArray):void {
            if (rect == null) {
                Error.throwError(TypeError, 2007, "rect");
            }
            if (destination == null) {
                Error.throwError(TypeError, 2007, "destination");
            }
        }

        [API("682")]
        public function copyToVector(rect:Rectangle, destination:Vector.<uint>):void {
            if (rect == null) {
                Error.throwError(TypeError, 2007, "rect");
            }
            if (destination == null) {
                Error.throwError(TypeError, 2007, "destination");
            }
        }

        [API("682")]
        public function drawToBitmapData(destination:BitmapData):void {
            if (destination == null) {
                Error.throwError(TypeError, 2007, "destination");
            }
        }

        public static function getCamera(name:String = null):Camera {
            return null;
        }

        public function setKeyFrameInterval(keyFrameInterval:int):void {
            this._keyFrameInterval = Math.max(1, Math.min(48, keyFrameInterval));
        }

        public function setLoopback(compress:Boolean = false):void {
            this._loopback = compress;
        }

        public function setMode(width:int, height:int, fps:Number, favorArea:Boolean = true):void {
            this._width = width;
            this._height = height;
            this._fps = fps;
        }

        public function setMotionLevel(motionLevel:int, timeout:int = 2000):void {
            this._motionLevel = motionLevel;
            this._motionTimeout = timeout;
        }

        public function setQuality(bandwidth:int, quality:int):void {
            this._bandwidth = bandwidth;
            this._quality = quality;
        }

        public function get activityLevel():Number {
            return -1;
        }

        public function get bandwidth():int {
            return this._bandwidth;
        }

        public function get currentFPS():Number {
            return 0;
        }

        public function get fps():Number {
            return this._fps;
        }

        public function get height():int {
            return this._height;
        }

        public function get index():int {
            return this._index;
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public function get keyFrameInterval():int {
            return this._keyFrameInterval;
        }

        public function get loopback():Boolean {
            return this._loopback;
        }

        public function get motionLevel():int {
            return this._motionLevel;
        }

        public function get motionTimeout():int {
            return this._motionTimeout;
        }

        public function get muted():Boolean {
            return true;
        }

        public function get name():String {
            return this._name;
        }

        public static function get names():Array {
            return [];
        }

        public function get quality():int {
            return this._quality;
        }

        public function get width():int {
            return this._width;
        }
    }
}

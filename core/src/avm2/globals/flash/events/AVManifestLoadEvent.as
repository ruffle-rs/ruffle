package flash.events {
    import flash.media.AVResult;

    [API("688")]
    public class AVManifestLoadEvent extends Event {
        public static const AV_MANIFEST_LOAD:String = "avManifestLoad";

        private var _result:AVResult;
        private var _userData:int;
        private var _handle:int;
        private var _duration:Number;

        public function AVManifestLoadEvent(type:String = "avManifestLoad", bubbles:Boolean = false, cancelable:Boolean = false, result:int = 0, userData:int = 0, handle:int = 0, duration:Number = 0.0) {
            super(type, bubbles, cancelable);
            this._result = new AVResult(result);
            this._userData = userData;
            this._handle = handle;
            this._duration = duration;
        }

        public function get result():AVResult {
            return this._result;
        }

        public function get userData():int {
            return this._userData;
        }

        public function get handle():int {
            return this._handle;
        }

        public function get duration():Number {
            return this._duration;
        }
    }
}

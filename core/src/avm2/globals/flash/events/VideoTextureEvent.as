package flash.events {
    public class VideoTextureEvent extends Event {
        [API("706")]
        public static const RENDER_STATE:String = "renderState";

        private var _status:String;
        private var _colorSpace:String;

        [API("706")]
        public const codecInfo:String;

        public function VideoTextureEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, status:String = null, colorSpace:String = null) {
            super(type, bubbles, cancelable);
            this._status = status;
            this._colorSpace = colorSpace;
        }

        [API("706")]
        public function get status():String {
            return this._status;
        }

        [API("706")]
        public function get colorSpace():String {
            return this._colorSpace;
        }
    }
}

package flash.events {
    public class VideoEvent extends Event {
        public const codecInfo:String;
        public static const RENDER_STATE:String = "renderState";
        public static const RENDER_STATUS_ACCELERATED:String = "accelerated";
        public static const RENDER_STATUS_SOFTWARE:String = "software";
        public static const RENDER_STATUS_UNAVAILABLE:String = "unavailable";

        private var _status:String;

        public function VideoEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, status:String = null) {
            super(type, bubbles, cancelable);
            this._status = status;
        }

        public function get status():String {
            return this._status;
        }
    }
}

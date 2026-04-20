package flash.events {
    // According to the AS3 docs, this class is only available starting with Flash Player 10.2,
    // and some members of it are AIR-only. This is yet another case of misdocumentation.
    [API("667")]
    public class StageVideoEvent extends Event {
        public static const RENDER_STATE:String = "renderState";
        public static const RENDER_STATUS_ACCELERATED:String = "accelerated";
        public static const RENDER_STATUS_SOFTWARE:String = "software";
        public static const RENDER_STATUS_UNAVAILABLE:String = "unavailable";

        public const codecInfo:String;

        private var _status:String;
        private var _colorSpace:String;

        public function StageVideoEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            status:String = null,
            colorSpace:String = null
        ) {
            super(type, bubbles, cancelable);
            this._status = status;
            this._colorSpace = colorSpace;
        }

        public function get status():String {
            return this._status;
        }

        public function get colorSpace():String {
            return this._colorSpace;
        }
    }
}

package flash.automation {
    import flash.events.Event;

    public class StageCaptureEvent extends Event {
        public static const CAPTURE:String = "capture";

        private var _url:String;
        private var _checksum:uint;
        private var _pts:Number;

        public function StageCaptureEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            url:String = "",
            checksum:uint = 0,
            pts:Number = 0
        ) {
            super(type, bubbles, cancelable);
            this._url = url;
            this._checksum = checksum;
            this._pts = pts;
        }

        public function get url():String {
            return this._url;
        }

        public function get checksum():uint {
            return this._checksum;
        }

        public function get pts():Number {
            return this._pts;
        }

        override public function clone():Event {
            return new StageCaptureEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.url,
                this.checksum
            );
        }

        override public function toString():String {
            return this.formatToString(
                "StageCaptureEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "url",
                "checksum"
            );
        }
    }
}

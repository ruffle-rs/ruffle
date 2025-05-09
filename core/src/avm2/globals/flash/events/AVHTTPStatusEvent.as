package flash.events {
    public class AVHTTPStatusEvent extends Event {
        public static const AV_HTTP_RESPONSE_STATUS:String = "avHttpResponseStatus";

        private var _status: int;
        private var _responseURL: String;
        private var _responseHeaders: Array;

        public function AVHTTPStatusEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, status:int = 0, responseUrl:String = null, responseHeaders:Array = null) {
            super(type, bubbles, cancelable);
            this._status = status;
            this.responseURL = responseUrl;
            this.responseHeaders = responseHeaders;
        }

        override public function clone():Event {
            return new AVHTTPStatusEvent(this.type, this.bubbles, this.cancelable, this.status, this.responseURL, this.responseHeaders);
        }

        override public function toString():String {
            return this.formatToString("AVHTTPStatusEvent", "type", "bubbles", "cancelable", "eventPhase", "status", "responseUrl", "responseHeaders");
        }

        public function get status():int {
            return this._status;
        }

        public function get responseURL():String {
            return this._responseURL;
        }
        public function set responseURL(value:String):void {
            this._responseURL = value;
        }

        public function get responseHeaders():Array {
            return this._responseHeaders;
        }
        public function set responseHeaders(value:Array):void {
            this._responseHeaders = value;
        }
    }
}

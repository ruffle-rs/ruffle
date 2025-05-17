package flash.events {
    public class HTTPStatusEvent extends Event {
        [API("661")]
        public static const HTTP_RESPONSE_STATUS:String = "httpResponseStatus";
        public static const HTTP_STATUS:String = "httpStatus";

        private var _status:int;
        private var _redirected:Boolean;
        private var _responseHeaders:Array;
        private var _responseURL:String;

        public function HTTPStatusEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, status:int = 0, redirected:Boolean = false) {
            super(type, bubbles, cancelable);
            this._status = status;
            this.redirected = redirected;
        }

        override public function clone():Event {
            return new HTTPStatusEvent(this.type, this.bubbles, this.cancelable, this.status, this.redirected);
        }

        override public function toString():String {
            return this.formatToString("HTTPStatusEvent", "type", "bubbles", "cancelable", "eventPhase", "status", "redirected", "responseURL");
        }

        public function get status():int {
            return this._status;
        }

        [API("690")]
        public function get redirected():Boolean {
            return this._redirected;
        }
        [API("690")]
        public function set redirected(value:Boolean):void {
            this._redirected = value;
        }

        [API("661")]
        public function get responseHeaders():Array {
            return this._responseHeaders;
        }
        [API("661")]
        public function set responseHeaders(value:Array):void {
            this._responseHeaders = value;
        }

        [API("661")]
        public function get responseURL():String {
            return this._responseURL;
        }
        [API("661")]
        public function set responseURL(value:String):void {
            this._responseURL = value;
        }
    }
}

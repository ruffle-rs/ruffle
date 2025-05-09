package flash.events {
    [API("667")]
    public class DRMAuthenticationErrorEvent extends ErrorEvent {
        public static const AUTHENTICATION_ERROR:String = "authenticationError";

        private var _subErrorID: int;
        private var _serverURL: String;
        private var _domain: String;

        public function DRMAuthenticationErrorEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, inDetail:String = "",
            inErrorID:int = 0, inSubErrorID:int = 0, inServerURL:String = null, inDomain:String = null) {
            super(type, bubbles, cancelable, inDetail, inErrorID);
            this.subErrorID = inSubErrorID;
            this.serverURL = inServerURL;
            this.domain = inDomain;
        }

        public function get subErrorID():int {
            return this._subErrorID;
        }
        public function set subErrorID(value:int):void {
            this._subErrorID = value;
        }

        public function get serverURL():String {
            return this._serverURL;
        }
        public function set serverURL(value:String):void {
            this._serverURL = value;
        }

        public function get domain():String {
            return this._domain;
        }
        public function set domain(value:String):void {
            this._domain = value;
        }

        override public function clone():Event {
            return new DRMAuthenticationErrorEvent(this.type, this.bubbles, this.cancelable, this.text, this.errorID, this.subErrorID, this.serverURL, this.domain);
        }
    }
}

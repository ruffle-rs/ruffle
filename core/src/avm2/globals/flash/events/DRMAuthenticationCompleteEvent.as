package flash.events {
    import flash.utils.ByteArray;

    [API("667")]
    public class DRMAuthenticationCompleteEvent extends Event {
        public static const AUTHENTICATION_COMPLETE:String = "authenticationComplete";

        private var _serverURL:String;
        private var _domain:String;
        private var _token:ByteArray;

        public function DRMAuthenticationCompleteEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            inServerURL:String = null,
            inDomain:String = null,
            inToken:ByteArray = null
        ) {
            super(type, bubbles, cancelable);
            this.serverURL = inServerURL;
            this.domain = inDomain;
            this.token = inToken;
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

        public function get token():ByteArray {
            return this._token;
        }
        public function set token(value:ByteArray):void {
            this._token = value;
        }

        override public function clone():Event {
            return new DRMAuthenticationCompleteEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.serverURL,
                this.domain,
                this.token
            );
        }
    }
}

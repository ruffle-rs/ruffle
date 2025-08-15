package flash.media {
    public class AVNetworkingParams {
        private var _forceNativeNetworking: Boolean;
        private var _readSetCookieHeader: Boolean;
        private var _useCookieHeaderForAllRequests: Boolean;
        private var _networkDownVerificationUrl: String;
        private var _appendRandomQueryParameter: String;

        public function AVNetworkingParams(
            init_forceNativeNetworking:Boolean = false,
            init_readSetCookieHeader:Boolean = true,
            init_useCookieHeaderForAllRequests:Boolean = false,
            init_networkDownVerificationUrl:String = ""
        ) {
            this._forceNativeNetworking = init_forceNativeNetworking;
            this._readSetCookieHeader = init_readSetCookieHeader;
            this._useCookieHeaderForAllRequests = init_useCookieHeaderForAllRequests;
            this._networkDownVerificationUrl = init_networkDownVerificationUrl;
        }

        public function get appendRandomQueryParameter():String {
            return this._appendRandomQueryParameter;
        }

        public function set appendRandomQueryParameter(value:String):void {
            this._appendRandomQueryParameter = value;
        }

        public function get forceNativeNetworking():Boolean {
            return this._forceNativeNetworking;
        }

        public function set forceNativeNetworking(value:Boolean):void {
            this._forceNativeNetworking = value;
        }

        public function get networkDownVerificationUrl():String {
            return this._networkDownVerificationUrl;
        }

        public function set networkDownVerificationUrl(value:String):void {
            this._networkDownVerificationUrl = value;
        }

        public function get readSetCookieHeader():Boolean {
            return this._readSetCookieHeader;
        }

        public function set readSetCookieHeader(value:Boolean):void {
            this._readSetCookieHeader = value;
        }

        public function get useCookieHeaderForAllRequests():Boolean {
            return this._useCookieHeaderForAllRequests;
        }

        public function set useCookieHeaderForAllRequests(value:Boolean):void {
            this._useCookieHeaderForAllRequests = value;
        }
    }
}

package flash.events {
    import flash.net.NetStream;

    [API("661")]
    public class DRMAuthenticateEvent extends Event {
        public static const DRM_AUTHENTICATE:String = "drmAuthenticate";
        public static const AUTHENTICATION_TYPE_DRM:String = "drm";
        public static const AUTHENTICATION_TYPE_PROXY:String = "proxy";

        private var _header:String;
        private var _userPrompt:String;
        private var _passPrompt:String;
        private var _urlPrompt:String;
        private var _authenticationType:String;
        private var _netstream:NetStream;

        public function DRMAuthenticateEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, header:String = "", userPrompt:String = "", passPrompt:String = "", urlPrompt:String = "", authenticationType:String = "", netstream:NetStream = null) {
            super(type, bubbles, cancelable);

            this._header = header;
            this._userPrompt = userPrompt;
            this._passPrompt = passPrompt;
            this._urlPrompt = urlPrompt;
            this._authenticationType = authenticationType;
            this._netstream = netstream;
        }

        public function get header():String {
            return this._header;
        }

        public function get usernamePrompt():String {
            return this._userPrompt;
        }

        public function get passwordPrompt():String {
            return this._passPrompt;
        }

        public function get urlPrompt():String {
            return this._urlPrompt;
        }

        public function get authenticationType():String {
            return this._authenticationType;
        }

        public function get netstream():NetStream {
            return this._netstream;
        }

        override public function clone():Event {
            return new DRMAuthenticateEvent(type, bubbles, cancelable, this._header, this._userPrompt, this._passPrompt, this._urlPrompt, this._authenticationType, this._netstream);
        }

        override public function toString():String {
            return this.formatToString("DRMAuthenticateEvent", "type", "bubbles", "cancelable", "eventPhase", "header", "usernamePrompt", "passwordPrompt", "urlPrompt", "authenticationType");
        }
    }
}


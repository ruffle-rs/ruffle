package flash.net.drm {
    [API("692")]
    public final class DRMDeviceGroup {
        private var _serverURL:String = null;
        private var _authenticationMethod:String = AuthenticationMethod.ANONYMOUS;
        private var _domain:String = null;
        private var _name:String = null;

        public function get serverURL():String {
            return this._serverURL;
        }

        public function get authenticationMethod():String {
            return this._authenticationMethod;
        }

        public function get domain():String {
            return this._domain;
        }

        public function get name():String {
            return this._name;
        }

        public function set name(value:String):void {
            this._name = value;
        }
    }
}

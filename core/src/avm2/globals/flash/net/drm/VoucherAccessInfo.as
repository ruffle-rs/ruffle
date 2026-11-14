package flash.net.drm {
    [API("667")]
    public final class VoucherAccessInfo {
        private var _displayName:String = null;
        private var _authenticationMethod:String = AuthenticationMethod.ANONYMOUS;
        private var _domain:String = null;
        private var _policyID:String = null;
        private var _deviceGroup:DRMDeviceGroup = null;

        public function get displayName():String {
            return this._displayName;
        }

        public function get authenticationMethod():String {
            return this._authenticationMethod;
        }

        public function get domain():String {
            return this._domain;
        }

        public function get policyID():String {
            return this._policyID;
        }

        public function get deviceGroup():DRMDeviceGroup {
            return this._deviceGroup;
        }
    }
}

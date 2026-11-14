package flash.net.drm {
    import flash.utils.ByteArray;

    [API("667")]
    public class DRMContentData {
        private var _serverURL:String = null;
        private var _authenticationMethod:String = AuthenticationMethod.ANONYMOUS;
        private var _licenseID:String = null;
        private var _domain:String = null;
        private var _voucherAccessInfo:Vector.<VoucherAccessInfo> = new Vector.<VoucherAccessInfo>();

        public function DRMContentData(rawData:ByteArray = null) {
            super();
        }

        public function get serverURL():String {
            return this._serverURL;
        }

        public function get authenticationMethod():String {
            return this._authenticationMethod;
        }

        public function get licenseID():String {
            return this._licenseID;
        }

        public function get domain():String {
            return this._domain;
        }

        public function getVoucherAccessInfo():Vector.<VoucherAccessInfo> {
            return this._voucherAccessInfo;
        }
    }
}

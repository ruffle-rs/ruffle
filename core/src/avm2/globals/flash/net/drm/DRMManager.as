package flash.net.drm {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;

    [API("667")]
    public final class DRMManager extends EventDispatcher {
        private static var _instance:DRMManager;
        private static var _networkIdleTimeout:Number = 120000;

        public function DRMManager() {
            super();
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public static function getDRMManager():DRMManager {
            if (_instance == null) {
                _instance = new DRMManager();
            }
            return _instance;
        }

        public static function get networkIdleTimeout():Number {
            return _networkIdleTimeout;
        }

        public static function set networkIdleTimeout(timeoutInMilliseconds:Number):void {
            _networkIdleTimeout = timeoutInMilliseconds;
        }

        public function authenticate(
            serverURL:String,
            domain:String,
            username:String,
            password:String
        ):void {
        }

        public function setAuthenticationToken(
            serverURL:String,
            domain:String,
            token:ByteArray
        ):void {
        }

        public function loadVoucher(
            contentData:DRMContentData,
            setting:String
        ):void {
        }

        public function loadPreviewVoucher(contentData:DRMContentData):void {
        }

        public function storeVoucher(voucher:ByteArray):void {
        }

        public function resetDRMVouchers():void {
        }

        [API("692")]
        public function addToDeviceGroup(
            deviceGroup:DRMDeviceGroup,
            forceRefresh:Boolean = false
        ):void {
        }

        [API("692")]
        public function removeFromDeviceGroup(deviceGroup:DRMDeviceGroup):void {
        }

        [API("690")]
        public function returnVoucher(
            inServerURL:String,
            immediateCommit:Boolean,
            licenseID:String,
            policyID:String
        ):void {
        }
    }
}

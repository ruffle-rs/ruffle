package flash.net.drm {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    import flash.utils.ByteArray;

    [API("667")]
    public class DRMContentData {
        public function DRMContentData(rawData:ByteArray = null) {
            super();
        }

        public function get serverURL():String {
            stub_getter("flash.net.drm.DRMContentData", "serverURL");

            return null;
        }

        public function get authenticationMethod():String {
            stub_getter("flash.net.drm.DRMContentData", "authenticationMethod");

            return null;
        }

        public function get licenseID():String {
            stub_getter("flash.net.drm.DRMContentData", "licenseID");

            return null;
        }

        public function get domain():String {
            stub_getter("flash.net.drm.DRMContentData", "domain");

            return null;
        }

        public function getVoucherAccessInfo():Vector.<VoucherAccessInfo> {
            stub_method("flash.net.drm.DRMContentData", "getVoucherAccessInfo");

            return null;
        }
    }
}

package flash.system {
    import flash.net.URLStream;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;

    [API("688")]
    public final class AuthorizedFeatures {
        public function createApplicationInstaller(strings:XML, icon:ByteArray):ApplicationInstaller {
            return new ApplicationInstaller();
        }

        public function enableDiskCache(stream:URLStream):Boolean {
            return false;
        }

        public function enableLocalSecurity():Boolean {
            return false;
        }

        public function enablePolling():Boolean {
            return false;
        }

        public function isFeatureEnabled(feature:String, data:String = null):Boolean {
            return false;
        }

        public function isNegativeToken():Boolean {
            return false;
        }
    }
}

package flash.system {
    import flash.utils.ByteArray;        
    import flash.events.EventDispatcher;

    [API("688")]
    public final class AuthorizedFeaturesLoader extends EventDispatcher {
        private var _authorizedFeatures:AuthorizedFeatures = null;

        public function AuthorizedFeaturesLoader() {
            super();
        }

        public function get authorizedFeatures():AuthorizedFeatures {
            return this._authorizedFeatures;
        }

        public function loadAuthorizedFeatures():void {
        }

        public function loadAuthorizedFeaturesFromData(data:ByteArray):void {
        }
    }
}

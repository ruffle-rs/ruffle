package flash.system {

    import flash.display.DisplayObjectContainer;
    import flash.system.ApplicationDomain;

    public class LoaderContext {
        public var allowCodeImport : Boolean;
        public var allowLoadBytesCodeExecution : Boolean;
        public var applicationDomain : ApplicationDomain;
        public var checkPolicyFile : Boolean;
        public var imageDecodingPolicy : String;
        public var parameters : Object; // unset by default
        public var requestedContentParent : DisplayObjectContainer; // unset by default
        public var securityDomain : SecurityDomain;

        public function LoaderContext(checkPolicyFile:Boolean = false, applicationDomain:ApplicationDomain = null, securityDomain:SecurityDomain = null) {
            this.allowCodeImport = true;
            this.allowLoadBytesCodeExecution = true;
            this.applicationDomain = applicationDomain;
            this.checkPolicyFile = checkPolicyFile;
            // This should be `ImageDecodingPolicy.ON_DEMAND;`, but that's an AIR only class.
            this.imageDecodingPolicy = "onDemand";
            this.securityDomain = securityDomain;
        }
    }
}

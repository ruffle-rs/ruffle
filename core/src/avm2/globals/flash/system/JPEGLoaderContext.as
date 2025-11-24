package flash.system {
    // Both the docs and playerglobal.swc says this is "663", but it's actually available in regular Flash Player
    [API("662")]
    public class JPEGLoaderContext extends LoaderContext {
        public var deblockingFilter:Number = 0.0;

        public function JPEGLoaderContext(
            deblockingFilter:Number = 0.0,
            checkPolicyFile:Boolean = false,
            applicationDomain:ApplicationDomain = null,
            securityDomain:SecurityDomain = null
        ) {
            super(checkPolicyFile, applicationDomain, securityDomain);
            this.deblockingFilter = deblockingFilter;
        }
    }
}

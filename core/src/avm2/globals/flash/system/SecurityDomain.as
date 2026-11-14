package flash.system {
    [Ruffle(Abstract)]
    public class SecurityDomain {
        public static native function get currentDomain():SecurityDomain;

        [API("682")]
        public function get domainID():String {
            return null;
        }
    }
}

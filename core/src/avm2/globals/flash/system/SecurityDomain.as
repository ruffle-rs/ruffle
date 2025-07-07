package flash.system {
    [Ruffle(InstanceAllocator)]
    public class SecurityDomain {
        private static var dummyDomain:SecurityDomain = instantiateInternal();

        private static native function instantiateInternal():SecurityDomain;

        public static function get currentDomain():SecurityDomain {
            return dummyDomain;
        }
    }
}

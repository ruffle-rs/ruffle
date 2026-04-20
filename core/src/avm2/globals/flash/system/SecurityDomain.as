package flash.system {
    [Ruffle(InstanceAllocator)]
    public class SecurityDomain {
        private static var dummyDomain:SecurityDomain = null;

        private static native function instantiateInternal():SecurityDomain;

        public static function get currentDomain():SecurityDomain {
            if (dummyDomain == null) {
                dummyDomain = instantiateInternal();
            }

            return dummyDomain;
        }
    }
}

package flash.system {
    import __ruffle__.stub_getter;

    import flash.utils.ByteArray;
    import flash.system.Worker;

    [API("680")] // the docs say 682, that's wrong
    [Ruffle(Abstract)]
    public final class WorkerDomain {
        public static function get isSupported():Boolean {
            return false;
        }

        private static var _current:WorkerDomain;

        public static function get current():WorkerDomain {
            stub_getter("flash.system.WorkerDomain", "current");

            if (!_current) {
                _current = instantiateInternal();
            }

            return _current;
        }

        public native function createWorker(swf:ByteArray, giveAppPrivileges:Boolean = false):Worker;

        private static native function instantiateInternal():WorkerDomain;
    }
}

package flash.system {
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
            if (!_current) {
                _current = instantiateInternal();
            }

            return _current;
        }

        public native function createWorker(swf:ByteArray, giveAppPrivileges:Boolean = false):Worker;

        public function listWorkers():Vector.<Worker> {
            return new <Worker>[Worker.current];
        }

        private static native function instantiateInternal():WorkerDomain;
    }
}

package flash.system {

    import flash.utils.ByteArray;
    import flash.system.Worker;
    import __ruffle__.stub_method;

    [API("680")] // the docs say 682, that's wrong
    public final class WorkerDomain {
        public static const isSupported:Boolean = false;
        
        private static var _current:WorkerDomain;
        
        public static function get current():WorkerDomain {
            if (!_current) {
                _current = new WorkerDomain();
            }
            return _current;
        }

        public function WorkerDomain() {}

        public function createWorker(swf:ByteArray, giveAppPrivileges:Boolean = false):Worker {
            stub_method("flash.system.WorkerDomain", "createWorker");

            return new Worker();
        }
    }
}

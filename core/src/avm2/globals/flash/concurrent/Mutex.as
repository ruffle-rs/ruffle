package flash.concurrent {
    [API("684")]
    public final class Mutex {
        public static function get isSupported():Boolean {
            return false;
        }

        public function Mutex() {
            if (!isSupported) {
                throw new Error("Error #1520: Mutex cannot be initialized.", 1520);
            }
        }

        public function lock():void {
        }

        public function tryLock():Boolean {
            return false;
        }

        public function unlock():void {
        }
    }
}

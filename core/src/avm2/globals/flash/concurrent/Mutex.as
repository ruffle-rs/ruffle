package flash.concurrent {
    [API("684")]
    public final class Mutex {
        public static function get isSupported():Boolean {
            return false;
        }
        
        public function Mutex() {
            throw new Error("Error #1520: Mutex cannot be initialized.", 1520);
        }
    }
}

package flash.system {
    import flash.events.EventDispatcher;

    [API("682")]
    public final class Worker extends EventDispatcher {
        public function Worker() {
            throw new ArgumentError("Error #2012: Worker$ class cannot be instantiated.", 2012);
        }

        public static function get isSupported():Boolean {
            return false;
        }
    }
}

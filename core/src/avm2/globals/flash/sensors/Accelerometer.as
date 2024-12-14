package flash.sensors {
    import flash.events.EventDispatcher;

    [API("667")]
    public class Accelerometer extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }

        public function setRequestedUpdateInterval(interval: Number) {
            __ruffle__.stub_method("flash.sensors.Accelerometer", "setRequestedUpdateInterval");
        }

        public function get muted(): Boolean {
            __ruffle__.stub_getter("flash.sensors.Accelerometer", "muted");
            return true;
        }
    }
}

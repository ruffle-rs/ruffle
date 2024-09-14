package flash.sensors {
    import flash.events.EventDispatcher;
    [API("667")]
    public class Accelerometer extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }
    }
}

package flash.sensors {
    import flash.events.EventDispatcher;
    [API("668")]
    public class Geolocation extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }
    }
}

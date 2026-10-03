package flash.sensors {
    import flash.events.EventDispatcher;

    [API("668")]
    public class Geolocation extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }

        public function setRequestedUpdateInterval(interval:Number) {
            __ruffle__.stub_method("flash.sensors.Geolocation", "setRequestedUpdateInterval");
        }

        public function get muted():Boolean {
            __ruffle__.stub_getter("flash.sensors.Geolocation", "muted");
            return true;
        }
    }
}

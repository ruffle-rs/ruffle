package flash.system {
    import flash.events.EventDispatcher;

    [API("682")]
    [Ruffle(Abstract)]
    public final class Worker extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }
    }
}

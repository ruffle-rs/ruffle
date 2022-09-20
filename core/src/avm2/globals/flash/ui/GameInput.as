package flash.ui {
    import flash.events.EventDispatcher;

    public final class GameInput extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }

        public static function get numDevices():int {
            return 0;
        }
    }
}
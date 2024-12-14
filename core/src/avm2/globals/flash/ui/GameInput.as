package flash.ui {
    import flash.events.EventDispatcher;

    [API("688")]
    public final class GameInput extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }

        public static function get numDevices():int {
            return 0;
        }

        public static function getDeviceAt(index:int):GameInputDevice {
            throw new RangeError("Error #1506: The specified range is invalid.", 1506);
        }
    }
}
package flash.events {
    import flash.ui.GameInputDevice;

    [API("688")] // the docs say 689 (AIR-only), that's wrong
    public final class GameInputEvent extends Event {
        public static const DEVICE_ADDED:String = "deviceAdded";
        public static const DEVICE_REMOVED:String = "deviceRemoved";
        public static const DEVICE_UNUSABLE:String = "deviceUnusable";

        private var _device:GameInputDevice;

        public function GameInputEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, device:GameInputDevice = null) {
            super(type, bubbles, cancelable);

            this._device = device;
        }

        public function get device():GameInputDevice {
            return this._device;
        }
    }
}

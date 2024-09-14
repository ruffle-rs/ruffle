package flash.events {
    [API("688")] // the docs say 689 (AIR-only), that's wrong
    public final class GameInputEvent extends Event {
        public static const DEVICE_ADDED:String = "deviceAdded";
        public static const DEVICE_REMOVED:String = "deviceRemoved";
        public static const DEVICE_UNUSABLE:String = "deviceUnusable";
    }
}
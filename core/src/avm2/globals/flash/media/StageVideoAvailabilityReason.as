package flash.media {
    [API("688")] // the docs say 670, that's wrong
    public final class StageVideoAvailabilityReason {
        public static const DRIVER_TOO_OLD:String = "driverTooOld";
        public static const NO_ERROR:String = "noError";
        public static const UNAVAILABLE:String = "unavailable";
        public static const USER_DISABLED:String = "userDisabled";
        public static const WMODE_INCOMPATIBLE:String = "wModeIncompatible";
    }
}

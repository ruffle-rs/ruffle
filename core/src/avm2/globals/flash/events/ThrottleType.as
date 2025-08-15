package flash.events
{

    [API("676")] // the docs say 674, that's wrong
    public class ThrottleType
    {
        // This constant is used for the status property in the ThrottleEvent class.
        public static const PAUSE:String = "pause";

        // This constant is used for the status property in the ThrottleEvent class.
        public static const RESUME:String = "resume";

        // This constant is used for the status property in the ThrottleEvent class.
        public static const THROTTLE:String = "throttle";

    }
}

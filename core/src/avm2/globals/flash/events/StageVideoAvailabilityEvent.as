package flash.events {
    public class StageVideoAvailabilityEvent extends Event {
        public static const STAGE_VIDEO_AVAILABILITY:String = "stageVideoAvailability";

        public const driver:String;
        public const reason:String;

        private var _availability:String;

        public function StageVideoAvailabilityEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            availability:String = null
        ) {
            super(type, bubbles, cancelable);
            this._availability = availability;
        }

        public function get availability():String {
            return this._availability;
        }
    }
}

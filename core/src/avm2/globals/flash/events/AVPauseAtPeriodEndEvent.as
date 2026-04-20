package flash.events {
    public class AVPauseAtPeriodEndEvent extends Event {
        public static const AV_PAUSE_AT_PERIOD_END:String = "avPauseAtPeriodEnd";

        private var _userData:int;

        public function AVPauseAtPeriodEndEvent(
            type:String = "avPauseAtPeriodEnd",
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            userData:int = 0
        ) {
            super(type, bubbles, cancelable);
            this._userData = userData;
        }

        public function get userData():int {
            return this._userData;
        }
    }
}

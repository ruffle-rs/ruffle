package flash.events {
    public class ActivityEvent extends Event {
        public static const ACTIVITY:String = "activity";

        private var _activating:Boolean;

        public function ActivityEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, activating:Boolean = false) {
            super(type, bubbles, cancelable);
            this.activating = activating;
        }

        public function get activating():Boolean {
            return this._activating;
        }
        public function set activating(value:Boolean):void {
            this._activating = value;
        }

        override public function clone() : Event {
            return new ActivityEvent(this.type, this.bubbles, this.cancelable, this.activating);
        }

        override public function toString(): String {
            return formatToString("ActivityEvent","type","bubbles","cancelable","eventPhase","activating");
        }
    }
}

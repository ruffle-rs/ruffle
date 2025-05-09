package flash.events {
    [API("676")] // the docs say 674, that's wrong
    public class ThrottleEvent extends Event {
        public static const THROTTLE:String = "throttle";

        private var _state:String;
        private var _targetFrameRate:Number;

        public function ThrottleEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, state:String = null, targetFrameRate:Number = 0) {
            super(type, bubbles, cancelable);
            this._state = state;
            this._targetFrameRate = targetFrameRate;
        }

        override public function clone():Event {
            return new ThrottleEvent(this.type, this.bubbles, this.cancelable, this.state, this.targetFrameRate);
        }

        override public function toString():String {
            return this.formatToString("ThrottleEvent", "type", "bubbles", "cancelable", "eventPhase", "state", "targetFrameRate");
        }

        public function get state():String {
            return this._state;
        }

        public function get targetFrameRate():Number {
            return this._targetFrameRate;
        }
    }
}

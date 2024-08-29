package flash.events
{
    [API("667")]
    public class PressAndTapGestureEvent extends GestureEvent
    {
        public static const GESTURE_PRESS_AND_TAP : String = "gesturePressAndTap";

        private var _tapLocalX: Number;
        private var _tapLocalY: Number;


        public function PressAndTapGestureEvent(type:String, bubbles:Boolean = true, cancelable:Boolean = false, phase:String = null,
                                                localX:Number = 0, localY:Number = 0, tapLocalX:Number = 0, tapLocalY:Number = 0,
                                                ctrlKey:Boolean = false, altKey:Boolean = false, shiftKey:Boolean = false,
                                                controlKey:Boolean = false) {
            super(type, bubbles, cancelable, phase, localX, localY, ctrlKey, altKey, shiftKey, controlKey);
            this._tapLocalX = tapLocalX;
            this._tapLocalY = tapLocalY;
        }

        override public function clone():Event {
            return new PressAndTapGestureEvent(this.type, this.bubbles, this.cancelable, this.phase, this.localX, this.localY,
                                               this.tapLocalX, this.tapLocalY, this.ctrlKey, this.altKey, this.shiftKey, this.controlKey);
        }

        override public function toString():String
        {
            // should fail on FP too, see discussion https://github.com/ruffle-rs/ruffle/pull/12330
            return this.formatToString("GestureEvent","type","bubbles","cancelable","args");
        }

        public function get tapLocalX(): Number {
            return this._tapLocalX;
        }

        public function set tapLocalX(value: Number): void {
            this._tapLocalX = value;
        }

        public function get tapLocalY(): Number {
            return this._tapLocalY;
        }

        public function set tapLocalY(value: Number): void {
            this._tapLocalY = value;
        }

        public native function get tapStageX():Number;
        public native function get tapStageY():Number;
    }
}

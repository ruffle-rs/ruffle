package flash.events
{
    public class TransformGestureEvent extends GestureEvent
    {
        public static const GESTURE_PAN : String = "gesturePan";
        public static const GESTURE_ROTATE : String = "gestureRotate";
        public static const GESTURE_SWIPE : String = "gestureSwipe";
        public static const GESTURE_ZOOM : String = "gestureZoom";

        private var _offsetX: Number;
        private var _offsetY: Number;
        private var _rotation: Number;
        private var _scaleX: Number;
        private var _scaleY: Number;
        private var _velocity: Number;


        public function TransformGestureEvent(type:String, bubbles:Boolean = true, cancelable:Boolean = false,
                                              phase:String = null, localX:Number = 0, localY:Number = 0,
                                              scaleX:Number = 1.0, scaleY:Number = 1.0,
                                              rotation:Number = 0, offsetX:Number = 0, offsetY:Number = 0,
                                              ctrlKey:Boolean = false, altKey:Boolean = false, shiftKey:Boolean = false,
                                              controlKey:Boolean = false, velocity:Number = 0) {
            super(type, bubbles, cancelable, phase, localX, localY, ctrlKey, altKey, shiftKey, controlKey);
            this._offsetX = offsetX
            this._offsetY = offsetY
            this._rotation = rotation
            this._scaleX = scaleX
            this._scaleY = scaleY
            this._velocity = velocity
        }

        override public function clone():Event {
            return new TransformGestureEvent(this.type, this.bubbles, this.cancelable, this.phase,
                                             this.localX, this.localY, this.scaleX, this.scaleY, this.rotation,
                                             this.offsetX, this.offsetY, this.ctrlKey, this.altKey, this.shiftKey,
                                             this.controlKey, this.velocity);
        }

        override public function toString():String
        {
            // should fail on FP too, see discussion https://github.com/ruffle-rs/ruffle/pull/12330
            return this.formatToString("TransformGestureEvent","type","bubbles","cancelable","args");
        }

        public function get offsetX(): Number {
            return this._offsetX;
        }

        public function set offsetX(value: Number): void {
            this._offsetX = value;
        }

        public function get offsetY(): Number {
            return this._offsetY;
        }

        public function set offsetY(value: Number): void {
            this._offsetY = value;
        }

        public function get rotation(): Number {
            return this._rotation;
        }

        public function set rotation(value: Number): void {
            this._rotation = value;
        }

        public function get scaleX(): Number {
            return this._scaleX;
        }

        public function set scaleX(value: Number): void {
            this._scaleX = value;
        }

        public function get scaleY(): Number {
            return this._scaleY;
        }

        public function set scaleY(value: Number): void {
            this._scaleY = value;
        }

        public function get velocity(): Number {
            return this._velocity;
        }

        public function set velocity(value: Number): void {
            this._velocity = value;
        }
    }
}
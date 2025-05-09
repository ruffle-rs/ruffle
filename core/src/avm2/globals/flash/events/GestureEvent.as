package flash.events {
    [API("667")]
    public class GestureEvent extends Event {
        public static const GESTURE_TWO_FINGER_TAP:String = "gestureTwoFingerTap";

        private var _phase:String;

        [Ruffle(NativeAccessible)]
        private var _localX:Number;

        [Ruffle(NativeAccessible)]
        private var _localY:Number;

        private var _ctrlKey:Boolean;
        private var _altKey:Boolean;
        private var _shiftKey:Boolean;
        private var _commandKey:Boolean;
        private var _controlKey:Boolean;

        public function GestureEvent(type:String, bubbles:Boolean = true, cancelable:Boolean = false, phase:String = null, localX:Number = 0,
            localY:Number = 0, ctrlKey:Boolean = false, altKey:Boolean = false, shiftKey:Boolean = false, commandKey:Boolean = false, controlKey:Boolean = false) {
            super(type, bubbles, cancelable);
            this.phase = phase;
            this.localX = localX;
            this.localY = localY;
            this.ctrlKey = ctrlKey;
            this.altKey = altKey;
            this.shiftKey = shiftKey;
            this.commandKey = commandKey;
            this.controlKey = controlKey;
        }

        public function get phase():String {
            return this._phase;
        }
        public function set phase(value:String):void {
            this._phase = value;
        }

        public function get localX():Number {
            return this._localX;
        }
        public function set localX(value:Number):void {
            this._localX = value;
        }

        public function get localY():Number {
            return this._localY;
        }
        public function set localY(value:Number):void {
            this._localY = value;
        }

        public function get ctrlKey():Boolean {
            return this._ctrlKey;
        }
        public function set ctrlKey(value:Boolean):void {
            this._ctrlKey = value;
        }

        public function get altKey():Boolean {
            return this._altKey;
        }
        public function set altKey(value:Boolean):void {
            this._altKey = value;
        }

        public function get shiftKey():Boolean {
            return this._shiftKey;
        }
        public function set shiftKey(value:Boolean):void {
            this._shiftKey = value;
        }

        // commandKey and controlKey are AIR-only

        [API("668")]
        public function get commandKey():Boolean {
            return this._commandKey;
        }
        [API("668")]
        public function set commandKey(value:Boolean):void {
            this._commandKey = value;
        }

        [API("668")]
        public function get controlKey():Boolean {
            return this._controlKey;
        }
        [API("668")]
        public function set controlKey(value:Boolean):void {
            this._controlKey = value;
        }

        override public function clone():Event {
            return new GestureEvent(this.type, this.bubbles, this.cancelable, this.phase, this.localX, this.localY, this.ctrlKey, this.altKey, this.shiftKey, this.commandKey, this.controlKey);
        }

        override public function toString():String {
            return this.formatToString("GestureEvent", "type", "bubbles", "cancelable", "phase", "localX", "localY", "stageX", "stageY", "ctrlKey", "altKey", "shiftKey");
        }

        public native function get stageX():Number;
        public native function get stageY():Number;
    }
}
